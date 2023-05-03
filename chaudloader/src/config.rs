use std::io::Write;

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct Config {
    pub enabled_mods: std::collections::BTreeSet<String>,
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
    Ok(std::fs::File::create(CONFIG_FILE_NAME)?.write_all(
        &toml::to_vec(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
    )?)
}
