pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: String,

    #[serde(default)]
    pub authors: Vec<String>,
}

pub struct State {
    trusted: bool,
    dlls: std::collections::HashMap<std::path::PathBuf, windows_libloader::ModuleHandle>,
}

impl State {
    pub fn new(trusted: bool) -> Self {
        Self {
            trusted,
            dlls: std::collections::HashMap::new(),
        }
    }

    pub fn is_trusted(&self) -> bool {
        self.trusted
    }

    pub fn add_dll(&mut self, name: std::path::PathBuf, dll: windows_libloader::ModuleHandle) {
        self.dlls.insert(name, dll);
    }
}
