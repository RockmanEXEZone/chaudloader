pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    #[serde(default)]
    pub authors: Vec<String>,
}
