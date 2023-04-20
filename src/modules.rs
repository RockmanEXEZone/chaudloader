use crate::dl;

pub static KERNEL32: std::sync::LazyLock<dl::ModuleHandle> =
    std::sync::LazyLock::new(|| unsafe { dl::ModuleHandle::get("kernel32.dll").unwrap() });

pub static D3D11: std::sync::LazyLock<dl::ModuleHandle> = std::sync::LazyLock::new(|| unsafe {
    dl::ModuleHandle::load(std::path::Path::new("C:\\windows\\system32\\d3d11.dll")).unwrap()
});
