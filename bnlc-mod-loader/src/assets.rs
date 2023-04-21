pub mod zipdat;

use crate::hooks;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct Replacer {
    replacements: std::collections::HashMap<std::path::PathBuf, tempfile::NamedTempFile>,
}

impl Replacer {
    fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            replacements: std::collections::HashMap::new(),
        })
    }

    pub fn add(&mut self, path: &std::path::Path) -> Result<impl WriteSeek, std::io::Error> {
        let dest_f = tempfile::NamedTempFile::new()?;
        log::info!(
            "replacing {} -> {}",
            path.display(),
            dest_f.path().display()
        );
        let dest_path = dest_f.path().to_path_buf();
        self.replacements.insert(path.to_path_buf(), dest_f);
        Ok(std::fs::File::create(dest_path)?)
    }

    pub fn get<'a>(&self, path: &'a std::path::Path) -> Result<ReplacedPath<'a>, anyhow::Error> {
        let src_file = if let Some(src_file) = self.replacements.get(path) {
            src_file
        } else {
            return Ok(ReplacedPath {
                replaced: false,
                path: std::borrow::Cow::Borrowed(path),
            });
        };

        // We do this silly two-phase copy thing because if we pass the original asset to BNLC it really doesn't like it.
        // It is not sufficient to pass the original temp file to BNLC with FILE_SHARE_DELETE, because for some reason we still can't delete it on process exit.
        // This is not the best way to do things, but it works, I guess.

        let _create_file_a_hook_guard =
            unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileAHook)? };
        let _create_file_w_hook_guard =
            unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileWHook)? };

        let mut src_f = std::fs::File::open(src_file.path())?;
        let mut dest_f = tempfile::NamedTempFile::new()?;

        std::io::copy(&mut src_f, &mut dest_f)?;
        let (_, path) = dest_f.keep()?;
        Ok(ReplacedPath {
            replaced: true,
            path: std::borrow::Cow::Owned(path),
        })
    }

    pub fn clear(&mut self) {
        self.replacements.clear();
    }
}

pub struct ReplacedPath<'a> {
    replaced: bool,
    path: std::borrow::Cow<'a, std::path::Path>,
}

impl<'a> ReplacedPath<'a> {
    pub fn is_replaced(&self) -> bool {
        self.replaced
    }
}

impl<'a> std::ops::Deref for ReplacedPath<'a> {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<'a> Drop for ReplacedPath<'a> {
    fn drop(&mut self) {
        if self.replaced {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

pub static REPLACER: std::sync::LazyLock<std::sync::Mutex<Replacer>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Replacer::new().unwrap()));
