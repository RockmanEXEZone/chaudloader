#![feature(lazy_cell)]

mod datfile;
mod dl;
mod dxgi_shim;
mod hooks;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: winapi::shared::minwindef::HINSTANCE,
    call_reason: winapi::shared::minwindef::DWORD,
    _reserved: winapi::shared::minwindef::LPVOID,
) -> winapi::shared::minwindef::BOOL {
    if call_reason != winapi::um::winnt::DLL_PROCESS_ATTACH {
        return winapi::shared::minwindef::TRUE;
    }
    hooks::stage0::install().unwrap();
    winapi::shared::minwindef::TRUE
}
