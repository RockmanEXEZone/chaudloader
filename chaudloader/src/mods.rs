pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: String,

    #[serde(default)]
    pub authors: Vec<String>,
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
