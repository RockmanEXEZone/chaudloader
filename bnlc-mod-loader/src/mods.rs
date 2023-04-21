pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: String,

    #[serde(default)]
    pub authors: Vec<String>,
}
