use crate::{
    data::Config,
    serializer::path::{CONFIG_FILE, DOTFILE_FOLDER},
};
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

// Serialize the configuration on disk
pub fn save_config(config: &Config) -> Result<()> {
    let file_path = PathBuf::from(format!("{}/{}", DOTFILE_FOLDER, CONFIG_FILE));
    let toml_string = toml::to_string_pretty(&config).context("Can't serialize")?;

    fs::write(&file_path, toml_string).context(format!("Can't write on file {:?}", file_path))?;

    Ok(())
}

// Deserialize the configuration on disk
pub fn load_config() -> Result<Config> {
    let file_path = PathBuf::from(format!("{}/{}", DOTFILE_FOLDER, CONFIG_FILE));
    let content =
        fs::read_to_string(&file_path).context(format!("Can't read file {:?}", file_path))?;
    let config: Config = toml::from_str(&content).context("Syntax error")?;

    Ok(config.into())
}
