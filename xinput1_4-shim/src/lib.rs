//! Contains shim functions that just forward xinput1_4.dll calls to the system xinput1_4.dll.
#![cfg(windows)]

static XINPUT1_4: std::sync::LazyLock<windows_libloader::ModuleHandle> =
    std::sync::LazyLock::new(|| unsafe {
        windows_libloader::ModuleHandle::load(
            &windows_libloader::get_system_directory().join("xinput1_4.dll"),
        )
        .unwrap()
    });

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    module: winapi::shared::minwindef::HINSTANCE,
    call_reason: winapi::shared::minwindef::DWORD,
    reserved: winapi::shared::minwindef::LPVOID,
) -> winapi::shared::minwindef::BOOL {
    match call_reason {
        winapi::um::winnt::DLL_PROCESS_ATTACH => {
            // This is technically not safe, as per https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices:
            // > You should never perform the following tasks from within DllMain:
            // > - Call LoadLibrary or LoadLibraryEx (either directly or indirectly). This can cause a deadlock or a crash.
            //
            // Unfortunately, we don't have a better choice, so we hope Microsoft doesn't break this later.
            //
            // But also whatever, everyone else is doing this: https://github.com/elishacloud/dxwrapper/blob/8ae3f626ea8fea500028c9e66abe79e8990ed478/Dllmain/Dllmain.cpp#L401
            std::mem::forget(
                windows_libloader::ModuleHandle::load(
                    &std::env::current_exe()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("chaudloader.dll"),
                )
                .unwrap(),
            );
        }
        _ => {}
    }

    // If this DLL is being attached, just load the underlying DLL which will have its own DllMain automatically called.
    std::sync::LazyLock::force(&XINPUT1_4);
    if call_reason == winapi::um::winnt::DLL_PROCESS_ATTACH {
        return winapi::shared::minwindef::TRUE;
    }

    type Func = unsafe extern "system" fn(
        module: winapi::shared::minwindef::HINSTANCE,
        call_reason: winapi::shared::minwindef::DWORD,
        reserved: winapi::shared::minwindef::LPVOID,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(XINPUT1_4.get_symbol_address("DllMain").unwrap())
    });
    ORIG(module, call_reason, reserved)
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetState(
    dw_user_index: winapi::shared::minwindef::DWORD,
    p_state: *mut winapi::um::xinput::XINPUT_STATE,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        p_state: *mut winapi::um::xinput::XINPUT_STATE,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(XINPUT1_4.get_symbol_address("XInputGetState").unwrap())
    });
    ORIG(dw_user_index, p_state)
}

#[no_mangle]
pub unsafe extern "system" fn XInputSetState(
    dw_user_index: winapi::shared::minwindef::DWORD,
    p_vibration: *mut winapi::um::xinput::XINPUT_VIBRATION,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        p_vibration: *mut winapi::um::xinput::XINPUT_VIBRATION,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(XINPUT1_4.get_symbol_address("XInputSetState").unwrap())
    });
    ORIG(dw_user_index, p_vibration)
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetCapabilities(
    dw_user_index: winapi::shared::minwindef::DWORD,
    dw_flags: winapi::shared::minwindef::DWORD,
    p_capabilities: *mut winapi::um::xinput::XINPUT_CAPABILITIES,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        dw_flags: winapi::shared::minwindef::DWORD,
        p_capabilities: *mut winapi::um::xinput::XINPUT_CAPABILITIES,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            XINPUT1_4
                .get_symbol_address("XInputGetCapabilities")
                .unwrap(),
        )
    });
    ORIG(dw_user_index, dw_flags, p_capabilities)
}

#[no_mangle]
pub unsafe extern "system" fn XInputEnable(
    enable: winapi::shared::minwindef::BOOL,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        enable: winapi::shared::minwindef::BOOL,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(XINPUT1_4.get_symbol_address("XInputEnable").unwrap())
    });
    ORIG(enable)
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetBatteryInformation(
    dw_user_index: winapi::shared::minwindef::DWORD,
    dev_type: winapi::shared::minwindef::BYTE,
    p_battery_information: *mut winapi::um::xinput::XINPUT_BATTERY_INFORMATION,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        dev_type: winapi::shared::minwindef::BYTE,
        p_battery_information: *mut winapi::um::xinput::XINPUT_BATTERY_INFORMATION,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            XINPUT1_4
                .get_symbol_address("XInputGetBatteryInformation")
                .unwrap(),
        )
    });
    ORIG(dw_user_index, dev_type, p_battery_information)
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetKeystroke(
    dw_user_index: winapi::shared::minwindef::DWORD,
    dw_reserved: winapi::shared::minwindef::DWORD,
    p_keystroke: winapi::um::xinput::PXINPUT_KEYSTROKE,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        dw_reserved: winapi::shared::minwindef::DWORD,
        p_keystroke: winapi::um::xinput::PXINPUT_KEYSTROKE,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(XINPUT1_4.get_symbol_address("XInputGetKeystroke").unwrap())
    });
    ORIG(dw_user_index, dw_reserved, p_keystroke)
}

#[no_mangle]
pub unsafe extern "system" fn XInputGetAudioDeviceIds(
    dw_user_index: winapi::shared::minwindef::DWORD,
    p_render_device_id: winapi::shared::ntdef::LPWSTR,
    p_render_count: *mut winapi::shared::minwindef::UINT,
    p_capture_device_id: winapi::shared::ntdef::LPWSTR,
    p_capture_count: *mut winapi::shared::minwindef::UINT,
) -> winapi::shared::minwindef::DWORD {
    type Func = unsafe extern "system" fn(
        dw_user_index: winapi::shared::minwindef::DWORD,
        p_render_device_id: winapi::shared::ntdef::LPWSTR,
        p_render_count: *mut winapi::shared::minwindef::UINT,
        p_capture_device_id: winapi::shared::ntdef::LPWSTR,
        p_capture_count: *mut winapi::shared::minwindef::UINT,
    ) -> winapi::shared::minwindef::DWORD;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            XINPUT1_4
                .get_symbol_address("XInputGetAudioDeviceIds")
                .unwrap(),
        )
    });
    ORIG(
        dw_user_index,
        p_render_device_id,
        p_render_count,
        p_capture_device_id,
        p_capture_count,
    )
}
