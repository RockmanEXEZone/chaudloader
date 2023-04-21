pub mod dat;

use crate::hooks;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct Replacer {
    replacers: std::collections::HashMap<
        std::path::PathBuf,
        Box<dyn Fn(&mut dyn ReadSeek, &mut dyn WriteSeek) -> Result<(), anyhow::Error> + Send>,
    >,
}

impl Replacer {
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

    pub fn get_replaced_path(
        &self,
        path: &std::path::Path,
    ) -> Result<Option<ReplacedPath>, anyhow::Error> {
        let replacer = if let Some(replacer) = self.replacers.get(path) {
            replacer
        } else {
            return Ok(None);
        };

        let _create_file_a_hook_guard =
            unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileAHook)? };
        let _create_file_w_hook_guard =
            unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileWHook)? };

        let mut src_f = std::fs::File::open(path)?;
        let mut dest_f = tempfile::NamedTempFile::new()?;
        replacer(&mut src_f, &mut dest_f)?;
        let (_, path) = dest_f.keep()?;
        Ok(Some(ReplacedPath(path)))
    }
}

pub struct ReplacedPath(std::path::PathBuf);

impl std::ops::Deref for ReplacedPath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ReplacedPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub static REPLACER: std::sync::LazyLock<std::sync::Mutex<Replacer>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Replacer::new().unwrap()));
