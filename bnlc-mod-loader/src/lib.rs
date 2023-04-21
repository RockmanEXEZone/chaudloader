#![feature(lazy_cell)]
#![feature(fs_try_exists)]

mod assets;
mod config;
mod hooks;
mod mods;

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
        winapi::um::winnt::DLL_PROCESS_DETACH => {
            // Destructors aren't run on termination, so we have to drop this ourselves to avoid lingering temporary files.
            let mut assets_replacer = assets::REPLACER.lock().unwrap();
            assets_replacer.clear();
        }
        _ => {}
    }
    winapi::shared::minwindef::TRUE
}
