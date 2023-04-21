pub mod mpak;
pub mod zipdat;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct Replacer {
    temp_dir: std::path::PathBuf,
    replacements: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
}

impl Replacer {
    pub fn new(game_name: &str) -> Result<Self, std::io::Error> {
        let temp_dir = std::env::temp_dir().join("bnlc_mod_loader").join(game_name);

        // Wipe existing temp directory, if possible.
        match std::fs::remove_dir_all(&temp_dir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(e);
            }
        }

        match std::fs::create_dir_all(&temp_dir) {
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
        let (dest_f, dest_path) = dest_f.keep()?;
        self.replacements.insert(path.to_path_buf(), dest_path);
        Ok(dest_f)
    }

    pub fn get<'a>(&'a self, path: &'a std::path::Path) -> (&'a std::path::Path, bool) {
        self.replacements
            .get(path)
            .map(|p| (p.as_path(), true))
            .unwrap_or((path, false))
    }
}

pub static REPLACER: std::sync::OnceLock<std::sync::Mutex<Replacer>> = std::sync::OnceLock::new();
