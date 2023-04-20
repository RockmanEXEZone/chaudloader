#![feature(lazy_cell)]

mod datfile;
mod hooks;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: winapi::shared::minwindef::HINSTANCE,
    call_reason: winapi::shared::minwindef::DWORD,
    _reserved: winapi::shared::minwindef::LPVOID,
) -> winapi::shared::minwindef::BOOL {
    match call_reason {
        winapi::um::winnt::DLL_PROCESS_ATTACH => {
            hooks::stage0::install().unwrap();
        }
        _ => {}
    }
    winapi::shared::minwindef::TRUE
}
