#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub mods: Vec<String>,
}
