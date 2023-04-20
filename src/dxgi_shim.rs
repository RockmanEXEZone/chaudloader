//! Contains shim functions that just forward dxgi.dll calls to the system dxgi.dll.
use crate::dl;

static DXGI: std::sync::LazyLock<dl::ModuleHandle> = std::sync::LazyLock::new(|| unsafe {
    dl::ModuleHandle::load(&dl::get_system_directory().join("dxgi.dll")).unwrap()
});

#[no_mangle]
pub unsafe extern "system" fn DXGIDumpJournal(
    lpv_unk_0: *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func =
        unsafe extern "system" fn(lpv_unk_0: *mut std::ffi::c_void) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("DXGIDumpJournal").unwrap())
    });
    ORIG(lpv_unk_0)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory(
    riid: winapi::shared::guiddef::REFIID,
    pp_factory: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        riid: winapi::shared::guiddef::REFIID,
        pp_factory: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("CreateDXGIFactory").unwrap())
    });
    ORIG(riid, pp_factory)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory1(
    riid: winapi::shared::guiddef::REFIID,
    pp_factory: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        riid: winapi::shared::guiddef::REFIID,
        pp_factory: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("CreateDXGIFactory1").unwrap())
    });
    ORIG(riid, pp_factory)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory2(
    flags: winapi::shared::minwindef::UINT,
    riid: winapi::shared::guiddef::REFIID,
    pp_factory: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        flags: winapi::shared::minwindef::UINT,
        riid: winapi::shared::guiddef::REFIID,
        pp_factory: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("CreateDXGIFactory2").unwrap())
    });
    ORIG(flags, riid, pp_factory)
}

#[no_mangle]
pub unsafe extern "system" fn DXGID3D10CreateDevice(
    handle: winapi::shared::ntdef::HANDLE,
    p_factory: *mut std::ffi::c_void,
    p_adapter: *mut std::ffi::c_void,
    flags: winapi::shared::minwindef::UINT,
    riid: winapi::shared::guiddef::REFIID,
    pp_device: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        handle: winapi::shared::ntdef::HANDLE,
        p_factory: *mut std::ffi::c_void,
        p_adapter: *mut std::ffi::c_void,
        flags: winapi::shared::minwindef::UINT,
        riid: winapi::shared::guiddef::REFIID,
        pp_device: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("DXGID3D10CreateDevice").unwrap())
    });
    ORIG(handle, p_factory, p_adapter, flags, riid, pp_device)
}

#[no_mangle]
pub unsafe extern "system" fn DXGID3D10CreateLayeredDevice(
    handle: winapi::shared::ntdef::HANDLE,
    flags: winapi::shared::minwindef::UINT,
    p_adapter: *mut std::ffi::c_void,
    riid: winapi::shared::guiddef::REFIID,
    pp_device: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        handle: winapi::shared::ntdef::HANDLE,
        flags: winapi::shared::minwindef::UINT,
        p_adapter: *mut std::ffi::c_void,
        riid: winapi::shared::guiddef::REFIID,
        pp_device: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            DXGI.get_symbol_address("DXGID3D10CreateLayeredDevice")
                .unwrap(),
        )
    });
    ORIG(handle, flags, p_adapter, riid, pp_device)
}

#[no_mangle]
pub unsafe extern "system" fn DXGID3D10GetLayeredDeviceSize(
    p_layer: *const std::ffi::c_void,
    num_layers: winapi::shared::minwindef::UINT,
) -> winapi::shared::basetsd::SIZE_T {
    type Func = unsafe extern "system" fn(
        p_layer: *const std::ffi::c_void,
        num_layers: winapi::shared::minwindef::UINT,
    ) -> winapi::shared::basetsd::SIZE_T;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            DXGI.get_symbol_address("DXGID3D10GetLayeredDeviceSize")
                .unwrap(),
        )
    });
    ORIG(p_layer, num_layers)
}

#[no_mangle]
pub unsafe extern "system" fn DXGID3D10RegisterLayers(
    layers: *const std::ffi::c_void,
    num_layers: winapi::shared::minwindef::UINT,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        layers: *const std::ffi::c_void,
        num_layers: winapi::shared::minwindef::UINT,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("DXGID3D10RegisterLayers").unwrap())
    });
    ORIG(layers, num_layers)
}

#[no_mangle]
pub unsafe extern "system" fn DXGIGetDebugInterface1(
    flags: winapi::shared::minwindef::UINT,
    riid: winapi::shared::guiddef::REFIID,
    p_debug: *mut *mut std::ffi::c_void,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        flags: winapi::shared::minwindef::UINT,
        riid: winapi::shared::guiddef::REFIID,
        p_debug: *mut *mut std::ffi::c_void,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(DXGI.get_symbol_address("DXGIGetDebugInterface1").unwrap())
    });
    ORIG(flags, riid, p_debug)
}

#[no_mangle]
pub unsafe extern "system" fn DXGIReportAdapterConfiguration(
    dw_unk_0: winapi::shared::minwindef::DWORD,
) -> winapi::um::winnt::HRESULT {
    type Func = unsafe extern "system" fn(
        dw_unk_0: winapi::shared::minwindef::DWORD,
    ) -> winapi::um::winnt::HRESULT;
    static ORIG: std::sync::LazyLock<Func> = std::sync::LazyLock::new(|| unsafe {
        std::mem::transmute(
            DXGI.get_symbol_address("DXGIReportAdapterConfiguration")
                .unwrap(),
        )
    });
    ORIG(dw_unk_0)
}
