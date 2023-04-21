#[derive(serde::Deserialize, serde::Serialize)]
pub struct ModConfig {
    pub name: String,

    #[serde(default)]
    pub trusted: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub mods: Vec<ModConfig>,
}
