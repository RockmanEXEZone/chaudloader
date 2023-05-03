use crate::{assets, gui, mods};
use retour::static_detour;
use std::os::windows::io::FromRawHandle;

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
) -> Result<std::collections::HashMap<String, assets::exedat::Overlay>, anyhow::Error> {
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
        let reader = assets::exedat::Reader::new(src_f)?;

        let overlay = assets::exedat::Overlay::new(reader);
        overlays.insert(file_name, overlay);
    }
    Ok(overlays)
}

struct Mod {
    info: mods::Info,
    readme: String,
    init_lua: String,
}

fn scan_mods() -> Result<std::collections::BTreeMap<String, Mod>, anyhow::Error> {
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
            let readme = std::fs::read_to_string(entry.path().join("README")).or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok("".to_string())
                } else {
                    Err(e)
                }
            })?;
            let init_lua: String = std::fs::read_to_string(entry.path().join("init.lua"))?;
            mods.insert(
                mod_name.to_string(),
                Mod {
                    info,
                    readme,
                    init_lua,
                },
            );
            Ok(())
        })() {
            log::warn!("[mod: {}] failed to load: {}", mod_name, e);
        }
    }
    Ok(mods)
}

struct HashWriter<T: std::hash::Hasher>(T);

impl<T: std::hash::Hasher> std::io::Write for HashWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf).map(|_| ())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn init(game_volume: crate::GameVolume) -> Result<(), anyhow::Error> {
    let console_reader = unsafe {
        let mut read_pipe = std::ptr::null_mut();
        let mut write_pipe = std::ptr::null_mut();
        assert_eq!(
            winapi::um::namedpipeapi::CreatePipe(
                &mut read_pipe,
                &mut write_pipe,
                std::ptr::null_mut(),
                0
            ),
            winapi::shared::minwindef::TRUE
        );
        assert_eq!(
            winapi::um::processenv::SetStdHandle(
                winapi::um::winbase::STD_OUTPUT_HANDLE,
                write_pipe
            ),
            winapi::shared::minwindef::TRUE
        );
        assert_eq!(
            winapi::um::processenv::SetStdHandle(winapi::um::winbase::STD_ERROR_HANDLE, write_pipe),
            winapi::shared::minwindef::TRUE
        );
        std::fs::File::from_raw_handle(read_pipe)
    };

    let (gui_host, mut gui_client) = gui::make_host_and_client();
    std::thread::spawn(move || {
        gui::run(gui_host, console_reader).unwrap();
        std::process::exit(0);
    });
    gui_client.wait_for_ready();

    env_logger::Builder::from_default_env()
        .filter(Some("chaudloader"), log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
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

    let exe_path = std::env::current_exe()?;
    let mut hasher = crc32fast::Hasher::new();
    {
        let mut exe_f = std::fs::File::open(&exe_path)?;
        std::io::copy(&mut exe_f, &mut HashWriter(&mut hasher))?;
    }

    let mod_env = mods::GameEnv {
        volume: game_volume,
        exe_crc32: hasher.finalize(),
    };

    // Load all archives as overlays.
    let overlays = scan_dats_as_overlays()?;
    let mut dat_names = overlays.keys().collect::<Vec<_>>();
    dat_names.sort_unstable();
    log::info!("found dat archives: {:?}", dat_names);

    let overlays = overlays
        .into_iter()
        .map(|(k, v)| (k, std::rc::Rc::new(std::cell::RefCell::new(v))))
        .collect::<std::collections::HashMap<_, _>>();

    // Scan for mods.
    let mods = scan_mods()?;
    let mod_names = mods.keys().collect::<Vec<_>>();
    log::info!("found mods: {:?}", mod_names);

    let mut loaded_mods = std::collections::HashMap::<String, mods::State>::new();

    for (mod_name, r#mod) in mods {
        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            if !r#mod.info.requires_loader_version.matches(&crate::VERSION) {
                return Err(anyhow::format_err!(
                    "version {} does not match requirement {}",
                    *crate::VERSION,
                    r#mod.info.requires_loader_version
                ));
            }

            log::info!(
                "[mod: {}] {} v{} by {}",
                mod_name,
                r#mod.info.title,
                r#mod.info.version,
                if !r#mod.info.authors.is_empty() {
                    r#mod.info.authors.join(", ")
                } else {
                    "(no authors listed)".to_string()
                }
            );

            let mod_state = std::rc::Rc::new(std::cell::RefCell::new(mods::State::new()));

            {
                let lua = mods::lua::new(
                    &mod_name,
                    &mod_env,
                    &r#mod.info,
                    std::rc::Rc::clone(&mod_state),
                    overlays.clone(),
                )?;
                lua.load(&r#mod.init_lua)
                    .set_name("=init.lua")
                    .set_mode(mlua::ChunkMode::Text)
                    .exec()?;
            }
            log::info!("[mod: {}] Lua script complete", mod_name);

            loaded_mods.insert(
                mod_name.to_string(),
                std::rc::Rc::try_unwrap(mod_state)
                    .map_err(|_| anyhow::anyhow!("mod_state: Rc was not unique"))
                    .unwrap()
                    .into_inner(),
            );

            Ok(())
        })() {
            log::error!("[mod: {}] failed to init: {}", mod_name, e);
        }
    }

    // We just need somewhere to keep LOADED_MODS so the DLLs don't get cleaned up, so we'll just put them here.
    std::thread_local! {
        static LOADED_MODS: std::cell::RefCell<
            Option<
                std::collections::HashMap<String, mods::State>,
            >,
        > = std::cell::RefCell::new(None);
    }
    LOADED_MODS.set(Some(loaded_mods));

    // We are done with mod initialization! We can now go repack everything from our overlays.
    {
        assert!(assets::REPLACER
            .set(std::sync::Mutex::new(assets::Replacer::new(
                &serde_plain::to_string(&game_volume).unwrap()
            )?))
            .is_ok());
        let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();

        let mut overlays = overlays;
        for (dat_filename, overlay) in overlays.drain() {
            let overlay = std::rc::Rc::try_unwrap(overlay)
                .map_err(|_| anyhow::anyhow!("overlay: Rc was not unique"))
                .unwrap()
                .into_inner();

            if !overlay.has_overlaid_files() {
                continue;
            }

            let overlay = std::cell::RefCell::new(overlay);

            // TODO: This path is a little wobbly, since it relies on BNLC specifying this weird relative path.
            // We should canonicalize this path instead.
            let dat_path = std::path::Path::new("..\\exe\\data").join(&dat_filename);
            assets_replacer.add(&dat_path, move |writer| {
                let mut overlay = overlay.borrow_mut();
                Ok(overlay.pack_into(writer)?)
            });
        }
    }
    unsafe {
        super::stage1::install()?;
    }
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

                        if let Some(game_volume) = match window_name.as_ref() {
                            "MegaMan_BattleNetwork_LegacyCollection_Vol1" => {
                                Some(crate::GameVolume::Vol1)
                            }
                            "MegaMan_BattleNetwork_LegacyCollection_Vol2" => {
                                Some(crate::GameVolume::Vol2)
                            }
                            _ => None,
                        } {
                            // Only initialize this once. It should be initialized on the main window being created.
                            static INITIALIZED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                                init(game_volume).unwrap();

                                let hwnd = CreateWindowExA.call(
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
                                );
                                if hwnd.is_null() {
                                    // This shouldn't happen...
                                    return hwnd;
                                }

                                assert_eq!(
                                    winapi::um::winuser::SetForegroundWindow(hwnd),
                                    winapi::shared::minwindef::TRUE
                                );

                                return hwnd;
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
