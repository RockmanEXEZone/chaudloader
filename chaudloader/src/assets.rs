use crate::hooks;

pub mod exedat;
pub mod mpak;
pub mod textarchive;

pub trait ReadSeek: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> ReadSeek for T {}

pub trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

enum Replacement {
    Pending(Box<dyn FnOnce(&mut dyn WriteSeek) -> Result<(), anyhow::Error> + Send>),
    Complete(std::path::PathBuf),
}

pub struct Replacer {
    temp_dir: std::path::PathBuf,
    replacements: std::collections::HashMap<std::path::PathBuf, Replacement>,
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
            replacements: std::collections::HashMap::new(),
        })
    }

    pub fn add(
        &mut self,
        path: &std::path::Path,
        pack_f: impl FnOnce(&mut dyn WriteSeek) -> Result<(), anyhow::Error> + Send + 'static,
    ) {
        self.replacements
            .insert(path.to_path_buf(), Replacement::Pending(Box::new(pack_f)));
    }

    pub fn get<'a>(
        &'a mut self,
        path: &'a std::path::Path,
    ) -> Result<(&'a std::path::Path, bool), anyhow::Error> {
        let replacement = if let Some(replacement) = self.replacements.get_mut(path) {
            replacement
        } else {
            return Ok((path, false));
        };

        match replacement {
            Replacement::Pending(_) => {
                let _create_file_a_hook_guard =
                    unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileAHook) }?;
                let _create_file_w_hook_guard =
                    unsafe { hooks::HookDisableGuard::new(&hooks::stage1::CreateFileWHook) }?;

                let dest_f = tempfile::NamedTempFile::new_in(&self.temp_dir)?;
                log::info!(
                    "replacing {} -> {}",
                    path.display(),
                    dest_f.path().display()
                );
                let (mut dest_f, dest_path) = dest_f.keep()?;

                let mut new_replacement = Replacement::Complete(dest_path);
                std::mem::swap(&mut new_replacement, replacement);

                let pack_f = match new_replacement {
                    Replacement::Pending(pack_f) => pack_f,
                    Replacement::Complete(_) => unreachable!(),
                };
                pack_f(&mut dest_f)?;

                Ok((
                    match replacement {
                        Replacement::Pending(_) => unreachable!(),
                        Replacement::Complete(p) => p.as_path(),
                    },
                    true,
                ))
            }
            Replacement::Complete(p) => Ok((p.as_path(), true)),
        }
    }
}

pub static REPLACER: std::sync::OnceLock<std::sync::Mutex<Replacer>> = std::sync::OnceLock::new();
