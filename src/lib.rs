#![feature(lazy_cell)]

mod d3d11_shim;
mod dl;
mod hooks;
mod modules;

fn main() -> Result<(), anyhow::Error> {
    unsafe {
        winapi::um::consoleapi::AllocConsole();
    }

    env_logger::Builder::from_default_env()
        .filter(Some("d3d11"), log::LevelFilter::Info)
        .init();
    log::info!("mod loader ready!");

    unsafe {
        hooks::install()?;
    }

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    _module: winapi::shared::minwindef::HINSTANCE,
    call_reason: winapi::shared::minwindef::DWORD,
    _reserved: winapi::shared::minwindef::LPVOID,
) -> winapi::shared::minwindef::BOOL {
    if call_reason != winapi::um::winnt::DLL_PROCESS_ATTACH {
        return winapi::shared::minwindef::TRUE;
    }
    main().unwrap();
    winapi::shared::minwindef::TRUE
}
