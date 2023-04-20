use crate::dl;

pub static KERNEL32: std::sync::LazyLock<dl::ModuleHandle> =
    std::sync::LazyLock::new(|| unsafe { dl::ModuleHandle::get("kernel32.dll").unwrap() });
