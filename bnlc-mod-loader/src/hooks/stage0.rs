use std::io::{Read, Seek, Write};

use crate::{assets, config, mods};
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
) -> Result<std::collections::HashMap<String, assets::dat::Overlay>, anyhow::Error> {
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
        let reader = assets::dat::Reader::new(src_f)?;

        let overlay = assets::dat::Overlay::new(reader);
        overlays.insert(file_name, overlay);
    }
    Ok(overlays)
}

fn scan_mods() -> Result<std::collections::HashMap<String, mods::Info>, anyhow::Error> {
    let mut mods = std::collections::HashMap::new();
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
            // Verify init.lua exists.
            if !std::fs::try_exists(entry.path().join("init.lua"))? {
                return Err(anyhow::anyhow!("missing init.lua"));
            }

            // Check for info.toml.
            let mut info_f = match std::fs::File::open(entry.path().join("info.toml")) {
                Ok(f) => f,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e.into());
                }
            };

            let mut buf = vec![];
            info_f.read_to_end(&mut buf)?;
            let info = toml::from_slice::<mods::Info>(&buf)?;

            mods.insert(mod_name.to_string(), info);

            Ok(())
        })() {
            log::warn!("[mod: {}] failed to load: {}", mod_name, e);
        }
    }
    Ok(mods)
}

unsafe fn init() -> Result<(), anyhow::Error> {
    winapi::um::consoleapi::AllocConsole();
    env_logger::Builder::from_default_env()
        .filter(Some("bnlc_mod_loader"), log::LevelFilter::Info)
        .init();
    log::info!("{}", BANNER);

    // Load config file, or create one if it doesn't exist.
    let config = {
        let mut config_f = std::fs::File::options()
            .create(true)
            .write(true)
            .read(true)
            .open("bnlc_mod_loader.toml")?;

        let mut buf = vec![];
        config_f.read_to_end(&mut buf)?;

        match toml::from_slice(&buf) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("failed to open bnlc_mod_loader.toml, will remake: {}", e);
                config_f.set_len(0)?;
                config_f.seek(std::io::SeekFrom::Start(0))?;
                let config = config::Config::default();
                config_f.write_all(toml::to_string(&config)?.as_bytes())?;
                config
            }
        }
    };

    // Load all archives as overlays.
    let overlays = scan_dats_as_overlays()?;
    let mut dat_names = overlays.keys().collect::<Vec<_>>();
    dat_names.sort_unstable();
    log::info!("found dat archives: {:?}", dat_names);
    let overlays = std::sync::Arc::new(std::sync::Mutex::new(overlays));

    // Scan for mods.
    let mods = scan_mods()?;
    let mut mod_names = mods.keys().collect::<Vec<_>>();
    mod_names.sort_unstable();
    log::info!("found mods: {:?}", mod_names);

    for mod_config in config.mods.iter() {
        if !mods.contains_key(&mod_config.name) {
            log::warn!(
                "mod {} was asked to load but it doesn't exist",
                mod_config.name
            );
        }

        let mod_path = std::path::Path::new("mods").join(&mod_config.name);

        let init_dll_path = {
            let path = mod_path.join("init.dll");
            if std::fs::try_exists(&path)? {
                if !mod_config.trusted {
                    log::warn!("[mod: {}] refusing to load because it was not marked as trusted but has a DLL entrypoint! if you want to load this mod, please add `trusted = true` to bnlc_mod_loader.toml for this mod!", mod_config.name);
                    continue;
                }
                Some(path)
            } else {
                None
            }
        };

        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            // Load Lua.
            let lua = mods::lua::new(&mod_config.name, std::sync::Arc::clone(&overlays))?;
            let mut init_f = std::fs::File::open(mod_path.join("init.lua"))?;
            let mut code = String::new();
            init_f.read_to_string(&mut code)?;
            lua.load(&code).exec()?;

            // Load DLL, if it exists.
            if let Some(init_dll_path) = init_dll_path {
                windows_libloader::ModuleHandle::load(&init_dll_path)
                    .ok_or_else(|| anyhow::anyhow!("DLL was requested to load but did not load"))?;
                log::info!(
                    "[mod: {}] DLL loaded. make sure you trust the authors of this mod!",
                    mod_config.name
                );
            }

            Ok(())
        })() {
            log::warn!("[mod: {}] failed to init: {}", mod_config.name, e);
        } else {
            log::info!("[mod: {}] initialized", mod_config.name);
        }
    }

    // We are done with mod initialization! We can now go repack everything from our overlays.
    {
        let mut assets_replacer = assets::REPLACER.lock().unwrap();
        let mut overlays = overlays.lock().unwrap();
        for (dat_filename, overlay) in overlays.drain() {
            let dat_path = std::path::Path::new("data").join(&dat_filename);

            let repacker = if let Some(repacker) = overlay.into_repacker()? {
                repacker
            } else {
                continue;
            };

            let mut writer = assets_replacer.add(&dat_path)?;
            repacker.pack_into(&mut writer)?;
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
                        if std::ffi::CStr::from_ptr(lp_window_name)
                            .to_string_lossy()
                            .starts_with("MegaMan_BattleNetwork_LegacyCollection_")
                        {
                            // Only initialize this once. It should be initialized on the main window being created.
                            static INITIALIZED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                                init().unwrap();
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
