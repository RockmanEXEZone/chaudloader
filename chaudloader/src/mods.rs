pub mod lua;

#[derive(serde::Deserialize)]
pub struct Info {
    pub title: String,

    pub version: semver::Version,

    #[serde(default)]
    pub requires_loader_version: semver::VersionReq,

    #[serde(default)]
    pub r#unsafe: bool,

    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Clone)]
pub struct GameEnv {
    pub volume: crate::GameVolume,
    pub exe_crc32: u32,
}

pub struct State {
    pub dlls: std::collections::HashMap<std::path::PathBuf, windows_libloader::ModuleHandle>,
}

impl State {
    pub fn new() -> Self {
        Self {
            dlls: std::collections::HashMap::new(),
        }
    }
}

pub struct Mod {
    pub info: Info,
    pub readme: String,
    pub init_lua: String,
}

pub fn scan() -> Result<std::collections::BTreeMap<String, Mod>, std::io::Error> {
    let mut mods = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir("mods")? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let path = entry.path();
        let mod_name = if let Some(mod_name) = path.file_name().unwrap().to_str() {
            mod_name
        } else {
            log::warn!("could not decipher mod name: {}", entry.path().display());
            continue;
        };

        if let Err(e) = (|| -> Result<(), anyhow::Error> {
            let info = toml::from_slice::<Info>(
                &std::fs::read(entry.path().join("info.toml")).map_err(|e| {
                    std::io::Error::new(
                        e.kind(),
                        anyhow::format_err!("error reading info.toml: {}", e),
                    )
                })?,
            )?;
            let readme = std::fs::read_to_string(entry.path().join("README.md")).or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok("".to_string())
                } else {
                    Err(std::io::Error::new(
                        e.kind(),
                        anyhow::format_err!("error reading README.md: {}", e),
                    ))
                }
            })?;
            let init_lua: String =
                std::fs::read_to_string(entry.path().join("init.lua")).map_err(|e| {
                    std::io::Error::new(
                        e.kind(),
                        anyhow::format_err!("error reading init.lua: {}", e),
                    )
                })?;
            mods.insert(
                mod_name.to_string(),
                Mod {
                    info,
                    readme,
                    init_lua,
                },
            );
            Ok(())
        })() {
            log::warn!("[mod: {}] failed to load: {}", mod_name, e);
        }
    }
    Ok(mods)
}
