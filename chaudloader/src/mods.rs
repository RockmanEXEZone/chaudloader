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

pub struct GameEnv {
    pub volume: crate::GameVolume,
    pub exe_crc32: u32,
}

pub struct State {
    pub dlls: std::collections::HashMap<std::path::PathBuf, windows_libloader::ModuleHandle>,
}

impl State {
    pub fn new() -> Self {
        Self {
            dlls: std::collections::HashMap::new(),
        }
    }
}
