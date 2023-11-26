use int_enum::IntEnum;
use std::io::Write;

// CHAUDLOADER CONFIG
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
    pub use_game_display_settings: Option<bool>,
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

// GAME CONFIG
#[repr(usize)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, IntEnum)]
pub enum ScreenMode {
    #[default]
    Windowed = 0,
    FullScreen = 1,
    Borderless = 2,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LauncherConfig {
    pub window_size: WindowSize,
    pub screen_mode: ScreenMode,
}

pub fn load_launcher_config() -> LauncherConfig {
    const LAUNCHERWINDOW_SIZES: &[WindowSize] = &[
        WindowSize {
            width: 1366,
            height: 768,
        },
        WindowSize {
            width: 1600,
            height: 900,
        },
        WindowSize {
            width: 1920,
            height: 1080,
        },
        WindowSize {
            width: 2560,
            height: 1440,
        },
        WindowSize {
            width: 3840,
            height: 2160,
        },
    ];
    let mut window_size = None;
    let mut screen_mode = None;
    if let Ok(launcher_config_ini) = std::fs::read_to_string("launcher_config.ini") {
        // Not actually .ini format :(
        for (k, v) in launcher_config_ini.lines().filter_map(|l| l.split_once(':')) {
            match k {
                "WindowSize" => {
                    if let Ok(Some(new_resolution)) =
                        v.parse::<usize>().map(|v| LAUNCHERWINDOW_SIZES.get(v))
                    {
                        window_size.get_or_insert(new_resolution);
                    }
                }
                "ScreenMode" => {
                    if let Ok(Ok(new_screen_mode)) =
                        v.parse::<usize>().map(|v| ScreenMode::from_int(v))
                    {
                        screen_mode.get_or_insert(new_screen_mode);
                    }
                }
                _ => {}
            }
        }
    }
    LauncherConfig {
        window_size: **window_size.get_or_insert(LAUNCHERWINDOW_SIZES.first().unwrap()), // TODO: Game uses screen resolution in this case
        screen_mode: screen_mode.unwrap_or_default(),
    }
}
