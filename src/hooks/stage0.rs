use crate::{datfile, dl};
use normpath::PathExt;
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

unsafe fn init() -> Result<(), anyhow::Error> {
    winapi::um::consoleapi::AllocConsole();
    env_logger::Builder::from_default_env()
        .filter(Some("bnlc_mod_loader"), log::LevelFilter::Info)
        .init();
    log::info!("hello!");

    let datfiles = std::fs::read_dir("data")?
        .map(|entry| {
            let entry = entry?;
            if entry.path().extension() != Some(&std::ffi::OsStr::new("dat")) {
                return Ok(None);
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("exe") && file_name != "reader.dat" && file_name != "rkb.dat"
            {
                return Ok(None);
            }

            let f = std::fs::File::open(entry.path())?;

            Ok::<_, anyhow::Error>(Some((file_name, datfile::Repacker::new(f)?)))
        })
        .flat_map(|v| match v {
            Ok(None) => None,
            Ok(Some(v)) => Some(Ok(v)),
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<std::collections::HashMap<_, _>, _>>()?;

    let mut datfile_names = datfiles.keys().collect::<Vec<_>>();
    datfile_names.sort_unstable();
    log::info!("loaded datfiles: {:?}", datfile_names);

    super::stage1::set_file_replacements(
        datfile_names
            .iter()
            .map(|file_name| {
                let path = std::path::Path::new(&format!("data/{}", file_name))
                    .normalize_virtually()
                    .unwrap()
                    .into_path_buf();
                (path.clone(), path)
            })
            .collect::<std::collections::HashMap<_, _>>(),
    );

    super::stage1::install()?;
    Ok(())
}

pub unsafe fn install() -> Result<(), anyhow::Error> {
    static USER32: std::sync::LazyLock<dl::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe { dl::ModuleHandle::get("user32.dll").unwrap() });

    unsafe {
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
