use crate::{assets, hooks, mods};
use retour::static_detour;

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

type CreateFileWFunc = unsafe extern "system" fn(
    lp_file_name: winapi::shared::ntdef::LPCWSTR,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE;

type CreateFileAFunc = unsafe extern "system" fn(
    lp_file_name: winapi::shared::ntdef::LPCSTR,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE;

static_detour! {
    static CreateFileWHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCWSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;

    static CreateFileAHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;
}

static_detour! {
    static mmbnlc_OnGameLoad: unsafe extern "system" fn(
        u32
    );
    static LoadFilePackage: unsafe extern "system" fn(
        /* this: */ *mut std::ffi::c_void,
        /* in_pszFilePackageName: */ *const winapi::shared::ntdef::WCHAR,
        /* out_uPackageID: */ *mut u32
    ) -> i32;
    static LoadBank: unsafe extern "system" fn(
        /* in_pszString: */ *const winapi::shared::ntdef::CHAR,
        /* out_bankID: */ *mut u32
    ) -> i32;
}

struct HooksDisableGuard {
    _create_file_w_guard: hooks::HookDisableGuard<CreateFileWFunc>,
    _create_file_a_guard: hooks::HookDisableGuard<CreateFileAFunc>,
}

impl HooksDisableGuard {
    unsafe fn new() -> Result<Self, retour::Error> {
        Ok(Self {
            _create_file_w_guard: hooks::HookDisableGuard::new(&CreateFileWHook)?,
            _create_file_a_guard: hooks::HookDisableGuard::new(&CreateFileAHook)?,
        })
    }
}

unsafe fn on_create_file(
    path: &std::path::Path,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE {
    let _hooks_disable_guard: HooksDisableGuard = HooksDisableGuard::new().unwrap();

    // FIXME: This path is relative to the exe folder, but is sometimes something like ..\exe\data\exe1.dat. We should canonicalize it in all cases to intercept all reads.
    let path = clean_path::clean(path);

    let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();
    let new_path = if let Some(new_path) = assets_replacer.get(&path).unwrap() {
        new_path
    } else {
        let path_wstr = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        return CreateFileWHook.call(
            path_wstr[..].as_ptr(),
            dw_desired_access,
            dw_share_mode,
            lp_security_attributes,
            dw_creation_disposition,
            dw_flags_and_attributes,
            handle,
        );
    };

    log::info!(
        "read to {} was redirected -> {}",
        path.display(),
        new_path.display()
    );

    let path_wstr = new_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    CreateFileWHook.call(
        path_wstr[..].as_ptr(),
        dw_desired_access,
        dw_share_mode,
        lp_security_attributes,
        dw_creation_disposition,
        dw_flags_and_attributes,
        handle,
    )
}

unsafe fn on_game_load(game_version: u32, gba_state: *mut u8) {
    mmbnlc_OnGameLoad.call(game_version);
    let mod_funcs = mods::MODFUNCTIONS.get().unwrap().lock().unwrap();
    for on_game_load_functions in &mod_funcs.on_game_load_functions {
        on_game_load_functions(game_version, gba_state);
    }
}

unsafe fn on_pck_load(
    sound_engine_class: *mut std::ffi::c_void,
    pck_file_name: *const winapi::shared::ntdef::WCHAR,
    out_pck_id: *mut u32,
) -> i32 {
    let pck_wstr = std::ffi::OsString::from_wide(std::slice::from_raw_parts(
        pck_file_name,
        winapi::um::winbase::lstrlenW(pck_file_name) as usize,
    ));

    let return_val = LoadFilePackage.call(sound_engine_class, pck_file_name, out_pck_id);
    match pck_wstr.to_str() {
        Some("Vol1.pck") | Some("Vol2.pck") => {
            // If both Vol1.pck and Vol2.pck are loaded, don't initialize again.
            static INITIALIZED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                let mod_pcks = mods::MODAUDIOFILES
                    .get()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .pcks
                    .clone();
                for pck in &mod_pcks {
                    let mod_pck_wstr = pck
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect::<Vec<_>>();
                    let mod_pck_wstr_ptr = mod_pck_wstr.as_ptr();
                    let load_mod_pck_res =
                        LoadFilePackage.call(sound_engine_class, mod_pck_wstr_ptr, out_pck_id);
                    log::info!(
                        "{} LoadFilePackage Result: {}",
                        pck.to_str().unwrap(),
                        load_mod_pck_res
                    );
                }
            }
        }
        _ => (),
    }
    return return_val;
}

unsafe fn on_bnk_load(
    bnk_file_name: *const winapi::shared::ntdef::CHAR,
    out_bnk_id: *mut u32,
) -> i32 {
    let bnk_str = std::ffi::CStr::from_ptr(bnk_file_name)
        .to_string_lossy()
        .to_string();

    let return_val = LoadBank.call(bnk_file_name, out_bnk_id);
    match bnk_str.as_str() {
        "Vol1Global.bnk" | "Vol2Global.bnk" => {
            // If both Vol1Global.bnk and Vol2Global.bnk are loaded, don't initialize again.
            static INITIALIZED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                let mod_bnks = mods::MODAUDIOFILES
                    .get()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .bnks
                    .clone();
                for bnk in &mod_bnks {
                    let mod_bnk_cstr = std::ffi::CString::new(bnk.as_encoded_bytes());
                    if let Ok(mod_bnk_cstr) = mod_bnk_cstr {
                        let load_mod_bnk_res = LoadBank.call(mod_bnk_cstr.as_ptr(), out_bnk_id);
                        log::info!(
                            "{} BankLoad Result: {}",
                            bnk.to_str().unwrap(),
                            load_mod_bnk_res
                        );
                    }
                }
            }
        }
        _ => (),
    }
    return return_val;
}

/// Install hooks into the process.
pub unsafe fn install() -> Result<(), anyhow::Error> {
    static KERNELBASE: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernelbase.dll").unwrap()
        });

    // BNLC actually uses both CreateFileA and CreateFileW... It seems like the third-party code uses CreateFileW but the BNLC code itself uses CreateFileA...
    //
    // Since we don't really care about the distincton, let's just normalize it here and hook it all via on_create_file.
    unsafe {
        CreateFileWHook
            .initialize(
                std::mem::transmute(KERNELBASE.get_symbol_address("CreateFileW").unwrap()),
                {
                    move |lp_file_name,
                          dw_desired_access,
                          dw_share_mode,
                          lp_security_attributes,
                          dw_creation_disposition,
                          dw_flags_and_attributes,
                          handle| {
                        on_create_file(
                            &std::path::PathBuf::from(std::ffi::OsString::from_wide(
                                std::slice::from_raw_parts(
                                    lp_file_name,
                                    winapi::um::winbase::lstrlenW(lp_file_name) as usize,
                                ),
                            )),
                            dw_desired_access,
                            dw_share_mode,
                            lp_security_attributes,
                            dw_creation_disposition,
                            dw_flags_and_attributes,
                            handle,
                        )
                    }
                },
            )?
            .enable()?;

        CreateFileAHook
            .initialize(
                std::mem::transmute(KERNELBASE.get_symbol_address("CreateFileA").unwrap()),
                {
                    move |lp_file_name,
                          dw_desired_access,
                          dw_share_mode,
                          lp_security_attributes,
                          dw_creation_disposition,
                          dw_flags_and_attributes,
                          handle| {
                        on_create_file(
                            std::path::Path::new(std::ffi::OsStr::new(
                                &std::ffi::CStr::from_ptr(lp_file_name)
                                    .to_string_lossy()
                                    .to_string(),
                            )),
                            dw_desired_access,
                            dw_share_mode,
                            lp_security_attributes,
                            dw_creation_disposition,
                            dw_flags_and_attributes,
                            handle,
                        )
                    }
                },
            )?
            .enable()?;
    }

    Ok(())
}

/// Install optional on_game_laod hook into the process.
pub unsafe fn install_on_game_load(game_env: &mods::GameEnv) -> Result<(), anyhow::Error> {
    unsafe {
        if let Some(data) = game_env.sections.text {
            // This pattern is enough to find the function in all releases of both collections (at 0x141dde120 Vol1 / 0x143147c10 Vol2 for latest releases)
            let on_game_load_pattern: [u8; 12] = [
                0x48, 0x89, 0x5c, 0x24, 0x10, 0x56, 0x48, 0x83, 0xec, 0x20, 0x8b, 0xd9,
            ];
            if let Some(offset) = data
                .windows(on_game_load_pattern.len())
                .position(|window| window == on_game_load_pattern)
            {
                let on_game_load_ptr = data.as_ptr().add(offset);
                // Get the offset to the GBAStruct from a structure referenced in the function
                let mov_instr_offset = 0x18;
                let struct_rel_offset = std::ptr::read_unaligned(
                    on_game_load_ptr.add(mov_instr_offset + 3) as *const u32,
                ) as usize;
                let struct_offset =
                    on_game_load_ptr.add(mov_instr_offset + 7 + struct_rel_offset) as u64;

                mmbnlc_OnGameLoad
                    .initialize(std::mem::transmute(on_game_load_ptr), {
                        move |game_version| {
                            // let gba_state = std::mem::transmute::<u64, * mut u8>(0x80200040);
                            // Get the gba state offset every time in case this struct moves
                            let struct_with_gba_state =
                                std::ptr::read_unaligned(struct_offset as *const *const u8);
                            let gba_state = std::ptr::read_unaligned(
                                struct_with_gba_state.add(0x3F8) as *const *mut u8,
                            );
                            on_game_load(game_version, gba_state)
                        }
                    })?
                    .enable()?;
            }
        }
    }
    Ok(())
}

/// Install optional PCK File load hook into the process.
pub unsafe fn install_pck_load(game_env: &mods::GameEnv) -> Result<(), anyhow::Error> {
    unsafe {
        if let Some(data) = game_env.sections.text {
            // This pattern is enough to find the function in all releases of both collections (at 0x14000A5C0 Vol1 / 0x14000BD20 Vol2 for the October 2023 releases)
            let pck_load_pattern: [u8; 24] = [
                0x40, 0x53, 0x55, 0x56, 0x57, 0x41, 0x56, 0x48, 0x81, 0xEC, 0x80, 0x00, 0x00, 0x00,
                0x48, 0xC7, 0x44, 0x24, 0x38, 0xFE, 0xFF, 0xFF, 0xFF, 0x48,
            ];
            if let Some(offset) = data
                .windows(pck_load_pattern.len())
                .position(|window| window == pck_load_pattern)
            {
                let pck_load_ptr = data.as_ptr().add(offset);
                LoadFilePackage
                    .initialize(std::mem::transmute(pck_load_ptr), {
                        move |sound_engine_class, pck_file_name, out_pck_id| {
                            on_pck_load(sound_engine_class, pck_file_name, out_pck_id)
                        }
                    })?
                    .enable()?;
            }
        }
    }
    Ok(())
}

/// Install optional BNK File load hook into the process.
pub unsafe fn install_bnk_load(game_env: &mods::GameEnv) -> Result<(), anyhow::Error> {
    unsafe {
        if let Some(data) = game_env.sections.text {
            // This pattern is enough to find the function in all releases of both collections (at 0x141CC27E0 Vol1 / 0x14302C310 Vol2 for the October 2023 releases)
            let bnk_load_pattern: [u8; 32] = [
                0x48, 0x89, 0x5C, 0x24, 0x08, 0x48, 0x89, 0x74, 0x24, 0x10, 0x48, 0x89, 0x7C, 0x24,
                0x18, 0x55, 0x48, 0x8D, 0x6C, 0x24, 0xA9, 0x48, 0x81, 0xEC, 0xE0, 0x00, 0x00, 0x00,
                0x48, 0x8B, 0xFA, 0x4C,
            ];
            if let Some(offset) = data
                .windows(bnk_load_pattern.len())
                .position(|window| window == bnk_load_pattern)
            {
                let bnk_load_ptr = data.as_ptr().add(offset);
                LoadBank
                    .initialize(std::mem::transmute(bnk_load_ptr), {
                        move |bnk_file_name, out_bnk_id| on_bnk_load(bnk_file_name, out_bnk_id)
                    })?
                    .enable()?;
            }
        }
    }
    Ok(())
}
