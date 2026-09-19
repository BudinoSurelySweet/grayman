use crate::{data::Config, serializer::data::CONFIG_FILE};
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

// Serialize the configuration on disk
pub fn save_config(mut config: Config) -> Result<()> {
    // If shell is not set than is true. Otherwise it'll has the specified value
    for task in &mut config.tasks {
        if let Some(shell) = task.shell
            && shell
        {
            task.shell = None;
        }
    }

    let file_path = PathBuf::from(CONFIG_FILE);
    let toml_string = toml::to_string_pretty(&config).context("Can't serialize")?;

    fs::write(&file_path, toml_string).context(format!("Can't write on file {:?}", file_path))?;

    Ok(())
}

// Deserialize the configuration on disk
pub fn load_config() -> Result<Config> {
    let file_path = PathBuf::from(CONFIG_FILE);
    let content =
        fs::read_to_string(&file_path).context(format!("Can't read file {:?}", file_path))?;
    let mut config: Config = toml::from_str(&content).context("Syntax error")?;

    // If shell is not set than is true. Otherwise it'll has the specified value
    for task in &mut config.tasks {
        if task.shell.is_none() {
            task.shell = Some(true);
        }
    }

    Ok(config)
}
