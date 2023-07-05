use crate::{assets, config, gui, mods};
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
    let (gui_host, mut gui_client) = gui::make_host_and_client();

    let mut config = config::load()?;

    let exe_crc32 = {
        let mut exe_f = std::fs::File::open(&std::env::current_exe()?)?;
        let mut hasher = crc32fast::Hasher::new();
        std::io::copy(&mut exe_f, &mut HashWriter(&mut hasher))?;
        hasher.finalize()
    };

    let game_env = mods::GameEnv {
        volume: game_volume,
        exe_crc32,
    };

    std::thread::spawn({
        let game_env = game_env.clone();
        let config = config.clone();
        move || {
            gui::run(gui_host, game_env, config).unwrap();
            std::process::exit(0);
        }
    });
    gui_client.wait_for_ready();

    env_logger::Builder::from_default_env()
        .filter(Some("chaudloader"), log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    // Make a mods directory if it doesn't exist.
    match std::fs::create_dir("mods") {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => {
            return Err(e.into());
        }
    };

    // Load all archives as overlays.
    let overlays = assets::exedat::scan()?;
    let mut dat_names = overlays.keys().collect::<Vec<_>>();
    dat_names.sort_unstable();

    let overlays = overlays
        .into_iter()
        .map(|(k, v)| (k, std::rc::Rc::new(std::cell::RefCell::new(v))))
        .collect::<std::collections::HashMap<_, _>>();

    let start_request = gui_client.wait_for_start();
    let enabled_mods = start_request
        .enabled_mods
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    log::info!("enabled mods: {:?}", enabled_mods);
    config.enabled_mods = enabled_mods;
    config.disable_autostart = start_request.disable_autostart;
    config::save(&config)?;

    let mut loaded_mods = std::collections::HashMap::<String, mods::State>::new();

    for (mod_name, r#mod) in start_request.enabled_mods {
        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            let compatibility = mods::check_compatibility(&game_env, &r#mod.info);
            if !compatibility.is_compatible() {
                return Err(anyhow::format_err!(
                    "compatibility not met: {:?}",
                    compatibility
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
                    &game_env,
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
