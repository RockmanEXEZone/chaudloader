pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: String,

    #[serde(default)]
    pub authors: Vec<String>,
}

pub struct State {
    init_dll: Option<windows_libloader::ModuleHandle>,
}

impl State {
    pub fn new() -> Self {
        Self { init_dll: None }
    }

    pub fn set_init_dll(&mut self, init_dll: windows_libloader::ModuleHandle) {
        self.init_dll = Some(init_dll);
    }
}
