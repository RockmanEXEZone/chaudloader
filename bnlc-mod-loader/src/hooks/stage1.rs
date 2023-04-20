use normpath::PathExt;
use retour::static_detour;

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

static_detour! {
    static CreateFileWHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCWSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;

    static CreateFileAHook: unsafe extern "system" fn(
        /* lp_file_name: */ winapi::shared::ntdef::LPCSTR,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* dw_share_mode: */ winapi::shared::minwindef::DWORD,
        /* lp_security_attributes: */ winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
        /* dw_creation_disposition: */ winapi::shared::minwindef::DWORD,
        /* dw_flags_and_attributes: */ winapi::shared::minwindef::DWORD,
        /* handle: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::ntdef::HANDLE;
}

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct AssetReplacer {
    replacers: std::collections::HashMap<
        std::path::PathBuf,
        Box<dyn Fn(&mut dyn ReadSeek, &mut dyn WriteSeek) -> Result<(), anyhow::Error> + Send>,
    >,
}

impl AssetReplacer {
    fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            replacers: std::collections::HashMap::new(),
        })
    }

    pub fn add(
        &mut self,
        name: &std::path::Path,
        replacer: impl Fn(&mut dyn ReadSeek, &mut dyn WriteSeek) -> Result<(), anyhow::Error>
            + Send
            + 'static,
    ) {
        self.replacers
            .insert(name.to_path_buf(), Box::new(replacer));
    }

    fn get_replaced_path(
        &self,
        path: &std::path::Path,
    ) -> Result<Option<std::path::PathBuf>, anyhow::Error> {
        let replacer = if let Some(replacer) = self.replacers.get(path) {
            replacer
        } else {
            return Ok(None);
        };

        unsafe {
            CreateFileAHook.disable()?;
            CreateFileWHook.disable()?;
        }

        let mut src_f = std::fs::File::open(path)?;
        let mut dest_f = tempfile::NamedTempFile::new()?;
        replacer(&mut src_f, &mut dest_f)?;

        unsafe {
            CreateFileAHook.enable()?;
            CreateFileWHook.enable()?;
        }

        let (_, path) = dest_f.keep()?;
        Ok(Some(path))
    }
}

pub static ASSET_REPLACER: std::sync::LazyLock<std::sync::Mutex<AssetReplacer>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(AssetReplacer::new().unwrap()));

unsafe fn on_create_file(
    path: &std::path::Path,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    mut dw_share_mode: winapi::shared::minwindef::DWORD,
    lp_security_attributes: winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: winapi::shared::minwindef::DWORD,
    dw_flags_and_attributes: winapi::shared::minwindef::DWORD,
    handle: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::ntdef::HANDLE {
    let path = path.normalize_virtually().unwrap().into_path_buf();

    let mut path = path
        .strip_prefix(std::env::current_dir().unwrap())
        .unwrap_or(&path)
        .to_path_buf();

    let asset_replacer = ASSET_REPLACER.lock().unwrap();
    let mut file_was_replaced = false;
    if let Some(replaced_path) = asset_replacer.get_replaced_path(&path).unwrap() {
        log::info!("read to {} was redirected", path.display());
        path = replaced_path;
        dw_share_mode |= winapi::um::winnt::FILE_SHARE_DELETE;
        file_was_replaced = true;
    }

    let path_wstr = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let h = CreateFileWHook.call(
        path_wstr[..].as_ptr(),
        dw_desired_access,
        dw_share_mode,
        lp_security_attributes,
        dw_creation_disposition,
        dw_flags_and_attributes,
        handle,
    );

    if file_was_replaced {
        std::fs::remove_file(path).unwrap();
    }

    h
}

/// Install hooks into the process.
pub unsafe fn install() -> Result<(), anyhow::Error> {
    static KERNEL32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernel32.dll").unwrap()
        });

    // BNLC actually uses both CreateFileA and CreateFileW... It seems like the third-party code uses CreateFileW but the BNLC code itself uses CreateFileA...
    //
    // Since we don't really care about the distincton, let's just normalize it here and hook it all via on_create_file.
    unsafe {
        CreateFileWHook
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("CreateFileW").unwrap()),
                {
                    move |lp_file_name,
                          dw_desired_access,
                          dw_share_mode,
                          lp_security_attributes,
                          dw_creation_disposition,
                          dw_flags_and_attributes,
                          handle| {
                        on_create_file(
                            &std::path::PathBuf::from(std::ffi::OsString::from_wide(
                                std::slice::from_raw_parts(
                                    lp_file_name,
                                    winapi::um::winbase::lstrlenW(lp_file_name) as usize,
                                ),
                            )),
                            dw_desired_access,
                            dw_share_mode,
                            lp_security_attributes,
                            dw_creation_disposition,
                            dw_flags_and_attributes,
                            handle,
                        )
                    }
                },
            )?
            .enable()?;

        CreateFileAHook
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("CreateFileA").unwrap()),
                {
                    move |lp_file_name,
                          dw_desired_access,
                          dw_share_mode,
                          lp_security_attributes,
                          dw_creation_disposition,
                          dw_flags_and_attributes,
                          handle| {
                        on_create_file(
                            std::path::Path::new(std::ffi::OsStr::new(
                                &std::ffi::CStr::from_ptr(lp_file_name)
                                    .to_string_lossy()
                                    .to_string(),
                            )),
                            dw_desired_access,
                            dw_share_mode,
                            lp_security_attributes,
                            dw_creation_disposition,
                            dw_flags_and_attributes,
                            handle,
                        )
                    }
                },
            )?
            .enable()?;
    }

    Ok(())
}
