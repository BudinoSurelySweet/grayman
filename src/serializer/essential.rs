use crate::{
    data::Config,
    serializer::path::{CONFIG_FILE, DOTFILE_FOLDER},
};
use anyhow::{Context, Result};
use std::{
    fs::{self, remove_dir_all},
    path::Path,
};

// Initialize all the necessary files in the current folder
pub fn init(force: bool) -> Result<()> {
    let dotfile_path = Path::new(DOTFILE_FOLDER);

    if dotfile_path.exists() && !force {
        return Err(anyhow::anyhow!("Faber is already initialized"));
    } else {
        fs::create_dir_all(dotfile_path).context(format!("Can't create {}", DOTFILE_FOLDER))?;
    }

    let config_path = dotfile_path.join(CONFIG_FILE);

    if !config_path.exists() {
        let config = Config::default();

        let toml_content =
            toml::to_string_pretty(&config).context("Can't serialize the default configuration")?;

        fs::write(&config_path, toml_content).context(format!("Can't create {}", CONFIG_FILE))?;
    }

    Ok(())
}

// Wipe all the files and folders of fucina
pub fn wipe() -> Result<()> {
    let path = Path::new(DOTFILE_FOLDER);

    if path.exists() {
        remove_dir_all(path)?;
    }

    Ok(())
}
