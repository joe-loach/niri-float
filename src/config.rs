use std::path::PathBuf;

/// Where the `niri-float` config directory lives.
pub fn config_dir() -> Option<PathBuf> {
    const CONFIG_DIR_NAME: &str = "niri-float";

    dirs::config_dir().map(|mut dir| {
        dir.push(CONFIG_DIR_NAME);
        dir
    })
}

/// Where the config file lives, inside of the config dir.
pub fn rules_path() -> Option<PathBuf> {
    const RULES_FILE: &str = "rules.toml";

    config_dir().map(|mut dir| {
        dir.push(RULES_FILE);
        dir
    })
}