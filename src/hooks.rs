use crate::modules;
use retour::static_detour;

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

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

unsafe fn create_file_w_hook(
    file_name: &std::path::Path,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE {
    log::info!("CreateFile: {}", file_name.display());
    let file_name_w = file_name
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    CreateFileWHook.call(
        file_name_w[..].as_ptr(),
        dw_desired_access,
        dw_share_mode,
        lp_security_attributes,
        dw_creation_disposition,
        dw_flags_and_attributes,
        handle,
    )
}

pub unsafe fn install() -> Result<(), anyhow::Error> {
    unsafe {
        CreateFileWHook
            .initialize(
                std::mem::transmute(modules::KERNEL32.get_symbol_address("CreateFileW").unwrap()),
                |lp_file_name,
                 dw_desired_access,
                 dw_share_mode,
                 lp_security_attributes,
                 dw_creation_disposition,
                 dw_flags_and_attributes,
                 handle| {
                    create_file_w_hook(
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
                },
            )?
            .enable()?;

        CreateFileAHook
            .initialize(
                std::mem::transmute(modules::KERNEL32.get_symbol_address("CreateFileA").unwrap()),
                |lp_file_name,
                 dw_desired_access,
                 dw_share_mode,
                 lp_security_attributes,
                 dw_creation_disposition,
                 dw_flags_and_attributes,
                 handle| {
                    create_file_w_hook(
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
                },
            )?
            .enable()?;
    }

    Ok(())
}
