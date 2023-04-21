mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub r#mod: ModInfo,
}

#[derive(serde::Deserialize)]
pub struct ModInfo {
    pub name: String,

    #[serde(default)]
    pub authors: Vec<String>,
}
