use crate::{
    assets, config, gui,
    mods::{self, MODAUDIOFILES, MODFUNCTIONS, ModAudioFiles, ModFunctions},
};
use byteorder::WriteBytesExt;
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
    static kernelbase_GetProcAddress: unsafe extern "system" fn(
        /* h_module: */ winapi::shared::minwindef::HMODULE,
        /* lp_proc_name: */ winapi::shared::ntdef::LPCSTR
    ) -> winapi::shared::minwindef::FARPROC;
}
static_detour! {
    static kernel32_GetProcAddress: unsafe extern "system" fn(
        /* h_module: */ winapi::shared::minwindef::HMODULE,
        /* lp_proc_name: */ winapi::shared::ntdef::LPCSTR
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
        sections: process_game_sections()
            .inspect_err(|e| log::warn!("error while processing game sections: {e}"))
            .unwrap_or_default(),
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
    assert!(
        assets::REPLACER
            .set(std::sync::Mutex::new(assets::Replacer::new(
                &serde_plain::to_string(&game_volume).unwrap()
            )?))
            .is_ok()
    );
    assert!(
        MODAUDIOFILES
            .set(std::sync::Mutex::new(ModAudioFiles::new()))
            .is_ok()
    );

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

    let on_game_load_hook_needed = init_mod_functions(&loaded_mods)?;

    let (pck_hook_needed, bnk_hook_needed) = init_mod_audio()?;

    // We just need somewhere to keep LOADED_MODS so the DLLs don't get cleaned up, so we'll just put them here.
    std::thread_local! {
        static LOADED_MODS: std::cell::RefCell<
            Option<
                std::collections::HashMap<String, mods::State>,
            >,
        > = const { std::cell::RefCell::new(None) };
    }
    LOADED_MODS.set(Some(loaded_mods));

    // We are done with mod initialization! We can now go repack everything from our overlays.
    {
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
                overlay.pack_into(writer)
            });
        }
    }
    unsafe {
        super::stage1::install()?;
        if on_game_load_hook_needed {
            super::stage1::install_on_game_load(&game_env)?;
        }
        if pck_hook_needed {
            super::stage1::install_pck_load(&game_env)?;
        }
        if bnk_hook_needed {
            super::stage1::install_bnk_load(&game_env)?;
        }
    }
    Ok(())
}

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

unsafe fn get_proc_address_hook(
    h_module: winapi::shared::minwindef::HMODULE,
    lp_proc_name: winapi::shared::ntdef::LPCSTR,
    config: &config::Config,
) -> winapi::shared::minwindef::FARPROC {
    #[derive(PartialEq)]
    enum HideState {
        Inactive,
        Active,
    }
    struct DevModeState {
        pub trigger_count: u64,
        pub hide_state: HideState,
        pub prev_proc_name: String,
    }
    impl DevModeState {
        pub fn new() -> Self {
            Self {
                trigger_count: 0,
                hide_state: HideState::Inactive,
                prev_proc_name: String::new(),
            }
        }
    }
    static DEV_MODE_STATE: std::sync::LazyLock<std::sync::Mutex<DevModeState>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(DevModeState::new()));

    let mut proc_address: usize =
        unsafe { kernelbase_GetProcAddress.call(h_module, lp_proc_name) as usize };

    // If we always do this, we crash for some reason, so just do it for relevant modules
    let proc_name = if h_module == KERNEL32.get_base_address()
        || h_module == KERNELBASE.get_base_address()
        || h_module == NTDLL.get_base_address()
    {
        unsafe {
            std::ffi::CStr::from_ptr(lp_proc_name)
                .to_str()
                .unwrap_or("")
        }
    } else {
        ""
    };

    if config.developer_mode == Some(true) && config.enable_hook_guards == Some(true) {
        let mut dev_mode_state = DEV_MODE_STATE.lock().unwrap();

        // disclaimer: ultra mega jank, likely will break in the future
        match dev_mode_state.hide_state {
            HideState::Inactive => {
                // Trigger on this call pattern:
                // GetProcAddress(kernel32, "GetProcAddress")
                // GetProcAddress(ntdll, ...)
                if h_module == NTDLL.get_base_address()
                    && dev_mode_state.prev_proc_name == "GetProcAddress"
                {
                    dev_mode_state.hide_state = HideState::Active;
                    dev_mode_state.trigger_count += 1;
                }
            }
            HideState::Active => {
                if h_module != NTDLL.get_base_address() {
                    dev_mode_state.hide_state = HideState::Inactive;
                }
            }
        }
        // If we return 0 on the first set of calls, we crash
        // Only do it on the second set of calls
        if dev_mode_state.hide_state == HideState::Active && dev_mode_state.trigger_count == 2 {
            // For some reason using std::ptr::null_mut() causes a crash when resuming from break?
            // So instead we directly mutate the pointer as an integer
            proc_address = 0;
        }

        dev_mode_state.prev_proc_name = proc_name.to_string();
    }

    // Avoid NtProtectVirtualMemory from being disabled
    if proc_name == "ZwProtectVirtualMemory" || proc_name == "NtProtectVirtualMemory" {
        proc_address = 0;
    }

    proc_address as winapi::shared::minwindef::FARPROC
}

pub unsafe fn install() -> Result<(), anyhow::Error> {
    static CONFIG: std::sync::LazyLock<config::Config> =
        std::sync::LazyLock::new(|| config::load().unwrap());

    if CONFIG.developer_mode == Some(true) {
        for cmd in CONFIG.stage0_commands.iter().flatten() {
            let (code, _, _) =
                run_script::run_script!(cmd.replace("%PID%", &std::process::id().to_string()))?;
            if code != 0 {
                log::error!("Command {cmd} exited with code {code}");
            }
        }
    }

    unsafe {
        // Hook GetProcAddress to avoid ntdll.dll hooks being installed.
        //
        // The function exists in both kernel32 and kernelbase; the one in kernel32 will call
        // kernelbase.GetProcAddressForCaller rather than kernelbase.GetProcAddress.
        // So we have to hook both of them to ensure our hook gets hit.
        // kernelbase.GetProcAddressForCaller doesn't exist in Wine/Proton, so we can't use that one.
        //
        // Hook the kernelbase variant first, because that's the one we call in our hook function.
        // Otherwise we will get infinite recursion!
        kernelbase_GetProcAddress
            .initialize(
                std::mem::transmute(KERNELBASE.get_symbol_address("GetProcAddress").unwrap()),
                {
                    move |h_module, lp_proc_name| {
                        get_proc_address_hook(h_module, lp_proc_name, &CONFIG)
                    }
                },
            )?
            .enable()?;
        kernel32_GetProcAddress
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("GetProcAddress").unwrap()),
                {
                    move |h_module, lp_proc_name| {
                        get_proc_address_hook(h_module, lp_proc_name, &CONFIG)
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
                        if !lp_window_name.is_null() {
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
                                if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst)
                                {
                                    init(game_volume, CONFIG.clone()).unwrap();

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

fn process_game_sections() -> Result<mods::Sections, anyhow::Error> {
    // Get sections of game executable
    let module = unsafe {
        windows_libloader::ModuleHandle::get(&std::env::current_exe()?.to_string_lossy())?
            .get_base_address() as *const u8
    };
    let sections = object::read::pe::PeFile64::parse(unsafe {
        std::slice::from_raw_parts(
            module, 0x1000, // probably enough
        )
    })?
    .section_table();

    // Make all sections read/write
    unsafe {
        for section in sections.iter() {
            let address = module.add(section.virtual_address.get(object::LittleEndian) as usize);
            if let Err(e) = region::protect(
                address,
                section.virtual_size.get(object::LittleEndian) as usize,
                region::Protection::READ_WRITE_EXECUTE,
            ) {
                log::warn!("Cannot unprotect section @ {:#?}: {e}", address);
            }
        }
    }

    // Return all recognized sections
    Ok(mods::Sections {
        // For text section, get the first section and check that it has the correct flags
        text: sections
            .section(1)
            .map_err(Into::into)
            .and_then(|s| {
                if (s.characteristics.get(object::LittleEndian)
                    & (object::pe::IMAGE_SCN_CNT_CODE
                        | object::pe::IMAGE_SCN_MEM_EXECUTE
                        | object::pe::IMAGE_SCN_MEM_READ))
                    != 0
                {
                    let (start, size) = s.pe_address_range();
                    Ok(unsafe {
                        std::slice::from_raw_parts(module.add(start as usize), size as usize)
                    })
                } else {
                    Err(anyhow::anyhow!("segment does not have correct flags"))
                }
            })
            .inspect_err(|e| log::error!("cannot find .text segment: {e}"))
            .ok(),
    })
}

fn init_mod_functions(
    loaded_mods: &std::collections::HashMap<String, mods::State>,
) -> Result<bool, anyhow::Error> {
    assert!(
        MODFUNCTIONS
            .set(std::sync::Mutex::new(ModFunctions::new()))
            .is_ok()
    );
    let mut on_game_load_hook_needed = false;
    let mut mod_funcs = MODFUNCTIONS.get().unwrap().lock().unwrap();
    for mod_state in loaded_mods.values() {
        for dll in mod_state.dlls.values() {
            unsafe {
                if let Ok(on_game_load_symbol_address) = dll.get_symbol_address("on_game_load") {
                    mod_funcs.on_game_load_functions.push(std::mem::transmute::<
                        winapi::shared::minwindef::FARPROC,
                        fn(u32, *const u8),
                    >(
                        on_game_load_symbol_address
                    ));
                    on_game_load_hook_needed = true;
                }
            }
        }
    }
    Ok(on_game_load_hook_needed)
}

fn init_mod_audio() -> Result<(bool, bool), anyhow::Error> {
    let mut mod_audio = MODAUDIOFILES.get().unwrap().lock().unwrap();
    let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();
    if !mod_audio.wems.is_empty() {
        let audio_path = std::path::Path::new("..\\exe\\audio\\chaudloader.pck");
        assets_replacer.add(audio_path, move |writer| generate_chaudloader_pck(writer));
        mod_audio
            .pcks
            .push(std::ffi::OsString::from("chaudloader.pck"));
    }
    // pcks.is_empty is checked here because pcks could either be added in the lua script or here if any wems were replaced in the lua script
    Ok((!mod_audio.pcks.is_empty(), !mod_audio.bnks.is_empty()))
}

fn generate_chaudloader_pck(
    mod_pck_file: &mut dyn assets::WriteSeek,
) -> Result<(), std::io::Error> {
    let mod_audio = MODAUDIOFILES.get().unwrap().lock().unwrap();
    // Generate chaudloader.pck from replacement wems
    let num_wem = mod_audio.wems.len() as u32;
    let mut wem_file_offset = 0x8C + num_wem * 20;
    // Write AKPK
    mod_pck_file.write_all(b"AKPK")?;
    // Write Pck header length
    mod_pck_file.write_u32::<byteorder::LittleEndian>(wem_file_offset)?;
    // Write next part of header
    mod_pck_file.write_all(&[
        0x01, 0x00, 0x00, 0x00, // PCK version
        0x68, 0x00, 0x00, 0x00, // Language Map Length
        0x04, 0x00, 0x00, 0x00, // Banks table Length
    ])?;
    // Write length of entries
    mod_pck_file.write_u32::<byteorder::LittleEndian>(num_wem * 20)?; // Stream table Length
    // Write next part of header
    mod_pck_file.write_all(&[
        0x04, 0x00, 0x00, 0x00, // "externalLUT" Length
        // Language Map
        0x04, 0x00, 0x00, 0x00, //Number of languages
        0x24, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, // Chinese offset + language ID
        0x34, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // English offset + language ID
        0x4C, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // Japanese offset + language ID
        0x5E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // SFX offset + language ID
        0x63, 0x00, 0x68, 0x00, 0x69, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x73, 0x00, 0x65, 0x00, 0x00,
        0x00, // "chinese" string
        0x65, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x6C, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68, 0x00, 0x28,
        0x00, 0x75, 0x00, 0x73, 0x00, 0x29, 0x00, 0x00, 0x00, // "english(us)" string
        0x6A, 0x00, 0x61, 0x00, 0x70, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x73, 0x00, 0x65,
        0x00, 0x00, 0x00, // "japanese" string
        0x73, 0x00, 0x66, 0x00, 0x78, 0x00, 0x00, 0x00, // "sfx" string
        0x00, 0x00, // padding
        // Banks table
        0x00, 0x00, 0x00, 0x00, // Number of files
    ])?;
    // Stream table
    // Write number of entries
    mod_pck_file.write_u32::<byteorder::LittleEndian>(num_wem)?;
    // IDs / Hashes need to be sorted in ascending order or the lookup fails
    let mut hashes: Vec<_> = mod_audio.wems.keys().cloned().collect();
    hashes.sort();
    // Skip entries and write wem files first
    mod_pck_file.seek(std::io::SeekFrom::Start(wem_file_offset as u64))?;
    // Write the actual wems and keep track and offsets and lengths
    let mut wem_offset_lens: Vec<(u32, u32)> = Vec::with_capacity(hashes.len());
    for hash in &hashes {
        let path = &mod_audio.wems.get(hash).unwrap().path;
        let wem_contents: Vec<u8> = std::fs::read(path)?;
        mod_pck_file.write_all(wem_contents.as_slice())?;
        wem_offset_lens.push((wem_file_offset, wem_contents.len() as u32));
        wem_file_offset += wem_contents.len() as u32;
    }
    // Go back to write the actual entries
    mod_pck_file.seek(std::io::SeekFrom::Start(0x8C))?;
    for (&hash, &(wem_offset, wem_size)) in hashes.iter().zip(wem_offset_lens.iter()) {
        let lanugage_id = mod_audio.wems.get(&hash).unwrap().language_id;
        mod_pck_file.write_u32::<byteorder::LittleEndian>(hash)?; // Hash / ID
        mod_pck_file.write_u32::<byteorder::LittleEndian>(0x01)?; // Block Size / required alignment
        mod_pck_file.write_u32::<byteorder::LittleEndian>(wem_size)?;
        mod_pck_file.write_u32::<byteorder::LittleEndian>(wem_offset)?;
        mod_pck_file.write_u32::<byteorder::LittleEndian>(lanugage_id)?; // Language ID
    }
    Ok(())
}
