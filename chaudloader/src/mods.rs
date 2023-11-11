pub mod lua;

#[derive(serde::Deserialize, Debug)]
pub struct Info {
    pub title: String,

    pub version: semver::Version,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub r#unsafe: bool,

    #[serde(default)]
    pub authors: Vec<String>,

    #[serde(default)]
    pub requires_loader_version: semver::VersionReq,

    #[serde(default)]
    pub requires_game: Option<std::collections::HashSet<crate::GameVolume>>,

    #[serde(default)]
    pub requires_exe_crc32: Option<std::collections::HashSet<u32>>,
}

#[derive(Clone, Default)]
pub struct Sections {
    pub text: Option<&'static [u8]>,
}

#[derive(Clone)]
pub struct GameEnv {
    pub volume: crate::GameVolume,
    pub exe_crc32: u32,
    pub sections: Sections,
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

#[derive(Debug)]
pub struct Compatibility {
    pub loader_version: bool,
    pub game: bool,
    pub exe_crc32: bool,
}

impl Compatibility {
    pub fn is_compatible(&self) -> bool {
        self.loader_version && self.game && self.exe_crc32
    }
}

pub fn check_compatibility(game_env: &GameEnv, info: &Info) -> Compatibility {
    Compatibility {
        loader_version: info.requires_loader_version.matches(&*crate::VERSION),
        game: info
            .requires_game
            .as_ref()
            .map(|games| games.contains(&game_env.volume))
            .unwrap_or(true),
        exe_crc32: info
            .requires_exe_crc32
            .as_ref()
            .map(|exe_crc32s| exe_crc32s.contains(&game_env.exe_crc32))
            .unwrap_or(true),
    }
}

pub fn scan() -> Result<std::collections::BTreeMap<String, std::sync::Arc<Mod>>, std::io::Error> {
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
                std::sync::Arc::new(Mod {
                    info,
                    readme,
                    init_lua,
                }),
            );
            Ok(())
        })() {
            log::warn!("[mod: {}] failed to load: {}", mod_name, e);
        }
    }
    Ok(mods)
}
