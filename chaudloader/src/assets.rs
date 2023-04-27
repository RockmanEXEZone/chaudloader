pub mod exedat;
pub mod mpak;
pub mod msg;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct Replacer {
    temp_dir: std::path::PathBuf,
    replacers: std::collections::HashMap<
        std::path::PathBuf,
        Box<dyn Fn(&mut dyn WriteSeek) -> Result<(), std::io::Error> + Send>,
    >,
    replacement_paths: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
}

impl Replacer {
    pub fn new(game_name: &str) -> Result<Self, std::io::Error> {
        let temp_dir = std::env::temp_dir().join("chaudloader").join(game_name);

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
            replacers: std::collections::HashMap::new(),
            replacement_paths: std::collections::HashMap::new(),
        })
    }

    pub fn add(
        &mut self,
        path: &std::path::Path,
        pack_cb: impl Fn(&mut dyn WriteSeek) -> Result<(), std::io::Error> + Send + 'static,
    ) {
        self.replacers.insert(path.to_path_buf(), Box::new(pack_cb));
    }

    pub fn get<'a>(
        &'a mut self,
        path: &std::path::Path,
    ) -> Result<Option<&'a std::path::Path>, std::io::Error> {
        Ok(match self.replacement_paths.entry(path.to_path_buf()) {
            std::collections::hash_map::Entry::Occupied(entry) => Some(entry.into_mut().as_path()),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let replacer = if let Some(replacer) = self.replacers.get(path) {
                    replacer
                } else {
                    return Ok(None);
                };

                let dest_path = self.temp_dir.join(std::path::Path::new(
                    &path
                        .as_os_str()
                        .to_string_lossy()
                        .replace("_", "__")
                        .replace("../", "_DOTDOTSLASH_")
                        .replace("./", "_DOTSLASH_")
                        .replace("/", "_SLASH_")
                        .replace("..\\", "_DOTDOTSLASH_")
                        .replace(".\\", "_DOTSLASH_")
                        .replace("\\", "_SLASH_"),
                ));
                {
                    let mut dest_f = std::fs::File::create(&dest_path)?;
                    log::info!("replacing {} -> {}", path.display(), dest_path.display());
                    replacer(&mut dest_f)?;
                }
                Some(entry.insert(dest_path).as_path())
            }
        })
    }
}

pub static REPLACER: std::sync::OnceLock<std::sync::Mutex<Replacer>> = std::sync::OnceLock::new();
