use std::io::Read;

use crate::{assets, mods};
use retour::static_detour;

static_detour! {
    static CreateWindowExA: unsafe extern "system" fn(
        /* dw_ex_style: */ winapi::shared::minwindef::DWORD,
        /* lp_class_name: */ winapi::shared::ntdef::LPCSTR,
        /* lp_window_name: */ winapi::shared::ntdef::LPCSTR,
        /* dw_style: */ winapi::shared::minwindef::DWORD,
        /* x: */ winapi::shared::minwindef::INT,
        /* y: */ winapi::shared::minwindef::INT,
        /* n_width: */ winapi::shared::minwindef::INT,
        /* n_height: */ winapi::shared::minwindef::INT,
        /* h_wnd_parent: */ winapi::shared::windef::HWND,
        /* h_menu: */ winapi::shared::windef::HMENU,
        /* h_instance: */ winapi::shared::minwindef::HINSTANCE,
        /* lp_param: */ winapi::shared::minwindef::LPVOID
    ) -> winapi::shared::windef::HWND;
}

const BANNER: &str = const_format::formatcp!(
    "

        %%%%%%%%%%%%%%%%%
     %%%%%  *********  %%%%%
   %%%% *************     %%%%
  %%% ***************       %%%
 %%% *************** ******* %%%
 %%% ************ ********** %%%    {}
 %%% ********** ************ %%%    v{}
 %%% ******* *************** %%%
  %%%       *************** %%%
   %%%%     ************* %%%%
     %%%%%  *********  %%%%%
        %%%%%%%%%%%%%%%%%
",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION")
);

fn scan_dats_as_overlays(
) -> Result<std::collections::HashMap<String, assets::zipdat::Overlay>, anyhow::Error> {
    let mut overlays = std::collections::HashMap::new();
    for entry in std::fs::read_dir("data")? {
        let entry = entry?;
        if entry.path().extension() != Some(&std::ffi::OsStr::new("dat")) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("exe") && file_name != "reader.dat" && file_name != "rkb.dat" {
            continue;
        }

        let src_f = std::fs::File::open(&entry.path())?;
        let reader = assets::zipdat::Reader::new(src_f)?;

        let overlay = assets::zipdat::Overlay::new(reader);
        overlays.insert(file_name, overlay);
    }
    Ok(overlays)
}

fn scan_mods() -> Result<std::collections::BTreeMap<String, mods::Info>, anyhow::Error> {
    let mut mods = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir("mods")? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let path = entry.path();
        let mod_name = path.file_name().unwrap().to_str().ok_or_else(|| {
            anyhow::anyhow!("could not decipher mod name: {}", entry.path().display())
        })?;

        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            let info =
                toml::from_slice::<mods::Info>(&std::fs::read(entry.path().join("info.toml"))?)?;
            mods.insert(mod_name.to_string(), info);
            Ok(())
        })() {
            log::warn!("[mod: {}] failed to load: {}", mod_name, e);
        }
    }
    Ok(mods)
}

unsafe fn init(game_name: &str) -> Result<(), anyhow::Error> {
    winapi::um::consoleapi::AllocConsole();
    env_logger::Builder::from_default_env()
        .filter(Some("chaudloader"), log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Never) // Under wine, this looks super broken, so let's just never write styles.
        .init();
    log::info!("{}", BANNER);

    // Make a mods directory if it doesn't exist.
    match std::fs::create_dir("mods") {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => {
            return Err(e.into());
        }
    };

    // Load all archives as overlays.
    let overlays = scan_dats_as_overlays()?;
    let mut dat_names = overlays.keys().collect::<Vec<_>>();
    dat_names.sort_unstable();
    log::info!("found dat archives: {:?}", dat_names);

    let overlays = overlays
        .into_iter()
        .map(|(k, v)| (k, std::sync::Arc::new(std::sync::Mutex::new(v))))
        .collect::<std::collections::HashMap<_, _>>();

    // Scan for mods.
    let mods = scan_mods()?;
    let mut mod_names = mods.keys().collect::<Vec<_>>();
    log::info!("found mods: {:?}", mod_names);

    let mut loaded_mods = std::collections::HashMap::<String, mods::State>::new();

    for (mod_name, mod_info) in mods {
        let mod_path = std::path::Path::new("mods").join(&mod_name);

        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            log::info!(
                "[mod: {}] {} v{} by {}",
                mod_name,
                mod_info.title,
                mod_info.version,
                if !mod_info.authors.is_empty() {
                    mod_info.authors.join(", ")
                } else {
                    "(no authors listed)".to_string()
                }
            );

            let mut mod_state = mods::State::new();

            // Load Lua, if it exists.
            match std::fs::File::open(mod_path.join("init.lua")) {
                Ok(mut init_f) => {
                    let lua = mods::lua::new(&mod_name, &mod_info, overlays.clone())?;
                    let mut code = String::new();
                    init_f.read_to_string(&mut code)?;
                    lua.load(&code).exec()?;
                    log::info!("[mod: {}] Lua script complete", mod_name);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {}
            }

            // Load DLL, if it exists.
            let init_dll_path = mod_path.join("init.dll");
            if std::fs::try_exists(&init_dll_path)? {
                let dll = windows_libloader::ModuleHandle::load(&init_dll_path)
                    .ok_or_else(|| anyhow::anyhow!("DLL was requested to load but did not load"))?;
                log::info!(
                    "[mod: {}] DLL loaded. make sure you trust the authors of this mod!",
                    mod_name
                );
                mod_state.set_init_dll(dll);
            }

            loaded_mods.insert(mod_name.to_string(), mod_state);

            Ok(())
        })() {
            log::warn!("[mod: {}] failed to init: {}", mod_name, e);
        }
    }

    static LOADED_MODS: std::sync::OnceLock<std::collections::HashMap<String, mods::State>> =
        std::sync::OnceLock::new();
    assert!(LOADED_MODS.set(loaded_mods).is_ok());

    // We are done with mod initialization! We can now go repack everything from our overlays.
    {
        assert!(assets::REPLACER
            .set(std::sync::Mutex::new(assets::Replacer::new(game_name)?))
            .is_ok());
        let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();

        let mut overlays = overlays;
        for (dat_filename, overlay) in overlays.drain() {
            let mut overlay = overlay.lock().unwrap();

            // TODO: This path is a little wobbly, since it relies on BNLC specifying this weird relative path.
            // We should canonicalize this path instead.
            let dat_path = std::path::Path::new("..\\exe\\data").join(&dat_filename);

            if !overlay.has_overlaid_files() {
                continue;
            }

            let mut writer = assets_replacer.add(&dat_path)?;
            overlay.pack_into(&mut writer)?;
        }
    }
    super::stage1::install()?;
    Ok(())
}

pub unsafe fn install() -> Result<(), anyhow::Error> {
    static USER32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("user32.dll").unwrap()
        });

    unsafe {
        // We hook CreateWindowExA specifically because BNLC may re-execute itself if not running under Steam. We don't want to go to all the trouble of repacking .dat files if we're just going to get re-executed.
        CreateWindowExA
            .initialize(
                std::mem::transmute(USER32.get_symbol_address("CreateWindowExA").unwrap()),
                {
                    move |dw_ex_style,
                          lp_class_name,
                          lp_window_name,
                          dw_style,
                          x,
                          y,
                          n_width,
                          n_height,
                          h_wnd_parent,
                          h_menu,
                          h_instance,
                          lp_param| {
                        let window_name =
                            std::ffi::CStr::from_ptr(lp_window_name).to_string_lossy();

                        if window_name.starts_with("MegaMan_BattleNetwork_LegacyCollection_") {
                            // Only initialize this once. It should be initialized on the main window being created.
                            static INITIALIZED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                                init(&window_name).unwrap();
                            } else {
                                log::warn!("initialization was attempted more than once?");
                            }
                        }

                        CreateWindowExA.call(
                            dw_ex_style,
                            lp_class_name,
                            lp_window_name,
                            dw_style,
                            x,
                            y,
                            n_width,
                            n_height,
                            h_wnd_parent,
                            h_menu,
                            h_instance,
                            lp_param,
                        )
                    }
                },
            )?
            .enable()?;
    }
    Ok(())
}
