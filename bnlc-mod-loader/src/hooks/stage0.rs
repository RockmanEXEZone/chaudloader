use crate::assets;
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

unsafe fn init() -> Result<(), anyhow::Error> {
    winapi::um::consoleapi::AllocConsole();
    env_logger::Builder::from_default_env()
        .filter(Some("bnlc_mod_loader"), log::LevelFilter::Info)
        .init();
    log::info!("{}", BANNER);

    {
        let mut asset_replacer = assets::REPLACER.lock().unwrap();

        for entry in std::fs::read_dir("data")? {
            let entry = entry?;
            if entry.path().extension() != Some(&std::ffi::OsStr::new("dat")) {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("exe") && file_name != "reader.dat" && file_name != "rkb.dat"
            {
                continue;
            }

            let mut src_f = std::fs::File::open(&entry.path())?;
            let repacker = assets::dat::Repacker::new(&mut src_f)?;
            let mut dest_f = asset_replacer.add(&entry.path())?;
            repacker.finish(&mut dest_f)?;
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
