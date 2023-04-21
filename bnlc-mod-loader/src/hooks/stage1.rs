use crate::assets;
use normpath::PathExt;
use retour::static_detour;

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

static_detour! {
    pub static CreateFileWHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCWSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;

    pub static CreateFileAHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;
}

unsafe fn on_create_file(
    path: &std::path::Path,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    mut dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE {
    let path = path.normalize_virtually().unwrap().into_path_buf();

    let path = path
        .strip_prefix(std::env::current_dir().unwrap())
        .unwrap_or(&path)
        .to_path_buf();

    let new_path = {
        let assets_replacer = assets::REPLACER.lock().unwrap();
        assets_replacer.get(&path).unwrap()
    };

    if new_path.is_replaced() {
        log::info!(
            "read to {} was redirected -> {}",
            path.display(),
            new_path.display()
        );
        // We set FILE_SHARE_DELETE here and delete the file immediately after CreateFileW to avoid temporary files hanging out.
        dw_share_mode |= winapi::um::winnt::FILE_SHARE_DELETE;
    }

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

/// Install hooks into the process.
pub unsafe fn install() -> Result<(), anyhow::Error> {
    static KERNEL32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernel32.dll").unwrap()
        });

    // BNLC actually uses both CreateFileA and CreateFileW... It seems like the third-party code uses CreateFileW but the BNLC code itself uses CreateFileA...
    //
    // Since we don't really care about the distincton, let's just normalize it here and hook it all via on_create_file.
    unsafe {
        CreateFileWHook
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("CreateFileW").unwrap()),
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
                std::mem::transmute(KERNEL32.get_symbol_address("CreateFileA").unwrap()),
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
