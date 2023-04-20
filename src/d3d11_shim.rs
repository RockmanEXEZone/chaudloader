use crate::modules;

type D3D11CreateDeviceAndSwapChainFunc = unsafe extern "system" fn(
    p_adapter: *mut winapi::shared::dxgi::IDXGIAdapter,
    driver_type: winapi::um::d3dcommon::D3D_DRIVER_TYPE,
    software: winapi::shared::minwindef::HMODULE,
    flags: winapi::shared::minwindef::UINT,
    p_feature_levels: *const winapi::um::d3dcommon::D3D_FEATURE_LEVEL,
    feature_levels: winapi::shared::minwindef::UINT,
    sdk_version: winapi::shared::minwindef::UINT,
    p_swap_chain_desc: *const winapi::shared::dxgi::DXGI_SWAP_CHAIN_DESC,
    pp_swap_chain: *mut *mut winapi::shared::dxgi::IDXGISwapChain,
    pp_device: *mut *mut winapi::um::d3d11::ID3D11Device,
    p_feature_level: *mut winapi::um::d3dcommon::D3D_FEATURE_LEVEL,
    pp_immediate_context: *mut *mut winapi::um::d3d11::ID3D11DeviceContext,
) -> winapi::um::winnt::HRESULT;

#[allow(non_upper_case_globals)]
pub static ORIG_D3D11CreateDeviceAndSwapChain: std::sync::LazyLock<
    D3D11CreateDeviceAndSwapChainFunc,
> = std::sync::LazyLock::new(|| unsafe {
    std::mem::transmute::<_, D3D11CreateDeviceAndSwapChainFunc>(
        modules::D3D11
            .get_symbol_address("D3D11CreateDeviceAndSwapChain")
            .unwrap(),
    )
});

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn D3D11CreateDeviceAndSwapChain(
    p_adapter: *mut winapi::shared::dxgi::IDXGIAdapter,
    driver_type: winapi::um::d3dcommon::D3D_DRIVER_TYPE,
    software: winapi::shared::minwindef::HMODULE,
    flags: winapi::shared::minwindef::UINT,
    p_feature_levels: *const winapi::um::d3dcommon::D3D_FEATURE_LEVEL,
    feature_levels: winapi::shared::minwindef::UINT,
    sdk_version: winapi::shared::minwindef::UINT,
    p_swap_chain_desc: *const winapi::shared::dxgi::DXGI_SWAP_CHAIN_DESC,
    pp_swap_chain: *mut *mut winapi::shared::dxgi::IDXGISwapChain,
    pp_device: *mut *mut winapi::um::d3d11::ID3D11Device,
    p_feature_level: *mut winapi::um::d3dcommon::D3D_FEATURE_LEVEL,
    pp_immediate_context: *mut *mut winapi::um::d3d11::ID3D11DeviceContext,
) -> winapi::um::winnt::HRESULT {
    let r = ORIG_D3D11CreateDeviceAndSwapChain(
        p_adapter,
        driver_type,
        software,
        flags,
        p_feature_levels,
        feature_levels,
        sdk_version,
        p_swap_chain_desc,
        pp_swap_chain,
        pp_device,
        p_feature_level,
        pp_immediate_context,
    );
    log::info!("D3D11CreateDeviceAndSwapChain called: {}", r);
    r
}
