#![feature(lazy_cell)]
#![feature(fs_try_exists)]
#![feature(local_key_cell_methods)]

mod assets;
mod hooks;
mod mods;

pub static VERSION: std::sync::LazyLock<semver::Version> =
    std::sync::LazyLock::new(|| semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap());

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
            // Maybe run destructors here, if we're feeling spicy.
        }
        _ => {}
    }
    winapi::shared::minwindef::TRUE
}
