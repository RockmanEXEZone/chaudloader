use std::io::Write;

pub const fn default_bool<const V: bool>() -> bool {
    V
}
pub const fn empty_btreeset<T>() -> std::collections::BTreeSet<T> {
    std::collections::BTreeSet::new()
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default = "default_bool::<false>")]
    pub disable_autostart: bool,
    #[serde(default = "empty_btreeset::<String>")]
    pub enabled_mods: std::collections::BTreeSet<String>,

    // Secret options
    pub developer_mode: Option<bool>,
    pub enable_hook_guards: Option<bool>, // requires developer_mode
    pub stage0_commands: Option<std::collections::BTreeSet<String>>, // requires developer_mode
}

const CONFIG_FILE_NAME: &str = "chaudloader.toml";

pub fn load() -> Result<Config, std::io::Error> {
    match std::fs::read(CONFIG_FILE_NAME) {
        Ok(b) => Ok(toml::from_slice(&b)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e),
    }
}

pub fn save(config: &Config) -> Result<(), std::io::Error> {
    std::fs::File::create(CONFIG_FILE_NAME)?.write_all(
        &toml::to_vec(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
    )
}
