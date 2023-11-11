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
static_detour! {
    static GetProcAddressForCaller: unsafe extern "system" fn(
        /* h_module: */ winapi::shared::minwindef::HMODULE,
        /* lp_proc_name: */ winapi::shared::ntdef::LPCSTR,
        /* lp_caller: */ winapi::shared::minwindef::LPVOID
    ) -> winapi::shared::minwindef::FARPROC;
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

fn init(
    game_volume: crate::GameVolume,
    mut config: crate::config::Config,
) -> Result<(), anyhow::Error> {
    let (gui_host, mut gui_client) = gui::make_host_and_client();

    let exe_crc32 = {
        let mut exe_f = std::fs::File::open(&std::env::current_exe()?)?;
        let mut hasher = crc32fast::Hasher::new();
        std::io::copy(&mut exe_f, &mut HashWriter(&mut hasher))?;
        hasher.finalize()
    };

    let game_env = mods::GameEnv {
        volume: game_volume,
        exe_crc32: exe_crc32,
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
    static KERNEL32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernel32.dll").unwrap()
        });
    static KERNELBASE: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernelbase.dll").unwrap()
        });
    static NTDLL: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("ntdll.dll").unwrap()
        });
    static USER32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("user32.dll").unwrap()
        });

    let config = config::load()?;

    if config.developer_mode == Some(true) {
        for cmd in config.stage0_commands.iter().flatten() {
            let (code, _, _) =
                run_script::run_script!(cmd.replace("%PID%", &std::process::id().to_string()))?;
            if code != 0 {
                log::error!("Command {cmd} exited with code {code}");
            }
        }
    }

    unsafe {
        // Hook GetProcAddress to avoid ntdll.dll hooks being installed
        GetProcAddressForCaller
            .initialize(
                std::mem::transmute(
                    KERNELBASE
                        .get_symbol_address("GetProcAddressForCaller")
                        .unwrap(),
                ),
                {
                    move |h_module, lp_proc_name, lp_caller| {
                        #[derive(PartialEq)]
                        enum HideState {
                            INACTIVE,
                            ACTIVE,
                        }
                        static mut TRIGGER_COUNT: u64 = 0;
                        static mut HIDE_STATE: HideState = HideState::INACTIVE;
                        static mut PREV_PROC_NAME: String = String::new();

                        let mut proc_address: usize =
                            GetProcAddressForCaller.call(h_module, lp_proc_name, lp_caller)
                                as usize;

                        // If we always do this, we crash for some reason, so just do it for relevant modules
                        let proc_name = if h_module == KERNEL32.get_base_address()
                            || h_module == NTDLL.get_base_address()
                        {
                            std::ffi::CStr::from_ptr(lp_proc_name)
                                .to_str()
                                .unwrap_or("")
                        } else {
                            ""
                        };

                        if config.developer_mode == Some(true)
                            && config.enable_hook_guards == Some(true)
                        {
                            // disclaimer: ultra mega jank, likely will break in the future
                            match &HIDE_STATE {
                                HideState::INACTIVE => {
                                    // Trigger on this call pattern:
                                    // GetProcAddress(kernel32, "GetProcAddress")
                                    // GetProcAddress(ntdll, ...)
                                    if h_module == NTDLL.get_base_address()
                                        && PREV_PROC_NAME == "GetProcAddress"
                                    {
                                        HIDE_STATE = HideState::ACTIVE;
                                        TRIGGER_COUNT += 1;
                                    }
                                }
                                HideState::ACTIVE => {
                                    if h_module != NTDLL.get_base_address() {
                                        HIDE_STATE = HideState::INACTIVE;
                                    }
                                }
                            }
                            // If we return 0 on the first set of calls, we crash
                            // Only do it on the second set of calls
                            if HIDE_STATE == HideState::ACTIVE && TRIGGER_COUNT == 2 {
                                // For some reason using std::ptr::null_mut() causes a crash when resuming from break?
                                // So instead we directly mutate the pointer as an integer
                                proc_address = 0;
                            }
                        }

                        // Avoid NtProtectVirtualMemory from being disabled
                        if proc_name == "ZwProtectVirtualMemory"
                            || proc_name == "NtProtectVirtualMemory"
                        {
                            proc_address = 0;
                        }

                        PREV_PROC_NAME = proc_name.to_string();
                        proc_address as winapi::shared::minwindef::FARPROC
                    }
                },
            )?
            .enable()?;

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
                                init(game_volume, config.clone()).unwrap();

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
