pub mod zipdat;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct Replacer {
    temp_dir: std::path::PathBuf,
    replacements: std::collections::HashMap<std::path::PathBuf, tempfile::NamedTempFile>,
}

impl Replacer {
    fn new() -> Result<Self, std::io::Error> {
        let temp_dir = std::env::temp_dir().join("bnlc_mod_loader");

        // Wipe existing temp directory, if possible.
        match std::fs::remove_dir_all(&temp_dir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(e);
            }
        }

        match std::fs::create_dir(&temp_dir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            temp_dir,
            replacements: std::collections::HashMap::new(),
        })
    }

    pub fn add(&mut self, path: &std::path::Path) -> Result<impl WriteSeek, std::io::Error> {
        let dest_f = tempfile::NamedTempFile::new_in(&self.temp_dir)?;
        log::info!(
            "replacing {} -> {}",
            path.display(),
            dest_f.path().display()
        );
        let dest_path = dest_f.path().to_path_buf();
        self.replacements.insert(path.to_path_buf(), dest_f);
        Ok(std::fs::File::create(dest_path)?)
    }

    pub fn get<'a>(&'a self, path: &'a std::path::Path) -> (&'a std::path::Path, bool) {
        self.replacements
            .get(path)
            .map(|p| (p.path(), true))
            .unwrap_or((path, false))
    }

    pub fn clear(&mut self) {
        self.replacements.clear();
    }
}

pub static REPLACER: std::sync::LazyLock<std::sync::Mutex<Replacer>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Replacer::new().unwrap()));
