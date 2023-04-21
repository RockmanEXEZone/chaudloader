use std::io::Read;

use crate::{assets, mods};
use retour::static_detour;

static_detour! {
    static CreateWindowExA: unsafe extern "system" fn(
        /* dw_ex_style: */ winapi::shared::minwindef::DWORD,
        /* lp_class_name: */ winapi::shared::ntdef::LPCSTR,
        /* lp_window_name: */ winapi::shared::ntdef::LPCSTR,
        /* dw_style: */ winapi::shared::minwindef::DWORD,
        /* x: */ winapi::shared::minwindef::INT,
        /* y: */ winapi::shared::minwindef::INT,
        /* n_width: */ winapi::shared::minwindef::INT,
        /* n_height: */ winapi::shared::minwindef::INT,
        /* h_wnd_parent: */ winapi::shared::windef::HWND,
        /* h_menu: */ winapi::shared::windef::HMENU,
        /* h_instance: */ winapi::shared::minwindef::HINSTANCE,
        /* lp_param: */ winapi::shared::minwindef::LPVOID
    ) -> winapi::shared::windef::HWND;
}

const BANNER: &str = const_format::formatcp!(
    "

        %%%%%%%%%%%%%%%%%
     %%%%%  *********  %%%%%
   %%%% *************     %%%%
  %%% ***************       %%%
 %%% *************** ******* %%%
 %%% ************ ********** %%%    {}
 %%% ********** ************ %%%    v{}
 %%% ******* *************** %%%
  %%%       *************** %%%
   %%%%     ************* %%%%
     %%%%%  *********  %%%%%
        %%%%%%%%%%%%%%%%%
",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION")
);

fn scan_mods() -> Result<std::collections::HashMap<std::ffi::OsString, mods::Info>, anyhow::Error> {
    match std::fs::read_dir("mods") {
        Ok(read_dir) => {
            let mut mods = std::collections::HashMap::new();
            for entry in read_dir {
                let entry = entry?;

                if !entry.file_type()?.is_dir() {
                    continue;
                }

                let mod_name = entry.path().file_name().unwrap().to_os_string();

                if let Err(e) = (|| -> Result<(), anyhow::Error> {
                    // Verify init.lua exists.
                    if !std::fs::try_exists(entry.path().join("init.lua"))? {
                        return Err(anyhow::anyhow!("missing init.lua"));
                    }

                    // Check for info.toml.
                    let mut info_f = match std::fs::File::open(entry.path().join("info.toml")) {
                        Ok(f) => f,
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    };

                    let mut buf = vec![];
                    info_f.read_to_end(&mut buf)?;
                    let info = toml::from_slice::<mods::Info>(&buf)?;

                    mods.insert(mod_name.clone(), info);

                    Ok(())
                })() {
                    log::warn!("[mod {}] failed to load: {}", mod_name.to_string_lossy(), e);
                }
            }
            Ok(mods)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::warn!("no mods directory found");
            Ok(std::collections::HashMap::new())
        }
        Err(e) => {
            return Err(e.into());
        }
    }
}

unsafe fn init() -> Result<(), anyhow::Error> {
    winapi::um::consoleapi::AllocConsole();
    env_logger::Builder::from_default_env()
        .filter(Some("bnlc_mod_loader"), log::LevelFilter::Info)
        .init();
    log::info!("{}", BANNER);

    // Scan for mods.
    let mods = scan_mods()?;

    let mut mod_names = mods.keys().collect::<Vec<_>>();
    mod_names.sort_unstable();
    log::info!("found mods: {:?}", mod_names);

    for (mod_name, info) in mods.iter() {
        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            let mod_ctx = mods::lua::Context::new(mod_name.clone())?;

            // Run any Lua code, if required.
            let mut init_f =
                std::fs::File::open(std::path::Path::new("mods").join(mod_name).join("init.lua"))?;
            let mut code = String::new();
            init_f.read_to_string(&mut code)?;
            mod_ctx.run(&code)?;

            Ok(())
        })() {
            log::warn!("[mod {}] failed to init: {}", mod_name.to_string_lossy(), e);
        } else {
            log::info!("[mod {}] initialized", mod_name.to_string_lossy());
        }
    }

    super::stage1::install()?;
    Ok(())
}

pub unsafe fn install() -> Result<(), anyhow::Error> {
    static USER32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("user32.dll").unwrap()
        });

    unsafe {
        // We hook CreateWindowExA specifically because BNLC may re-execute itself if not running under Steam. We don't want to go to all the trouble of repacking .dat files if we're just going to get re-executed.
        CreateWindowExA
            .initialize(
                std::mem::transmute(USER32.get_symbol_address("CreateWindowExA").unwrap()),
                {
                    move |dw_ex_style,
                          lp_class_name,
                          lp_window_name,
                          dw_style,
                          x,
                          y,
                          n_width,
                          n_height,
                          h_wnd_parent,
                          h_menu,
                          h_instance,
                          lp_param| {
                        if std::ffi::CStr::from_ptr(lp_window_name)
                            .to_string_lossy()
                            .starts_with("MegaMan_BattleNetwork_LegacyCollection_")
                        {
                            // Only initialize this once. It should be initialized on the main window being created.
                            static INITIALIZED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
                                init().unwrap();
                            } else {
                                log::warn!("initialization was attempted more than once?");
                            }
                        }

                        CreateWindowExA.call(
                            dw_ex_style,
                            lp_class_name,
                            lp_window_name,
                            dw_style,
                            x,
                            y,
                            n_width,
                            n_height,
                            h_wnd_parent,
                            h_menu,
                            h_instance,
                            lp_param,
                        )
                    }
                },
            )?
            .enable()?;
    }
    Ok(())
}
