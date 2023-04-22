pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: semver::Version,

    #[serde(default)]
    pub requires_loader_version: semver::VersionReq,

    #[serde(default)]
    pub r#unsafe: bool,

    #[serde(default)]
    pub authors: Vec<String>,
}

pub struct Env {
    pub game_volume: crate::GameVolume,
    pub exe_sha256: Vec<u8>,
}

pub struct State {
    dlls: std::collections::HashMap<std::path::PathBuf, windows_libloader::ModuleHandle>,
}

impl State {
    pub fn new() -> Self {
        Self {
            dlls: std::collections::HashMap::new(),
        }
    }

    pub fn add_dll(&mut self, path: std::path::PathBuf, dll: windows_libloader::ModuleHandle) {
        self.dlls.insert(path, dll);
    }
}
