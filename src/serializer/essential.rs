use crate::{
    data::Config,
    serializer::data::{CONFIG_FILE, DOTFILE_FOLDER, GITIGNORE_CONTENT},
};
use anyhow::{Context, Result};
use std::{
    fs::{self, remove_dir_all, remove_file},
    io::Write,
    path::Path,
};

// Initialize all the necessary files for grayman in the current folder
pub fn init(force: bool) -> Result<()> {
    let dotfile_path = Path::new(DOTFILE_FOLDER);

    // Initialize the dotfile folder
    if dotfile_path.exists() && !force {
        return Err(anyhow::anyhow!("Faber is already initialized"));
    } else {
        fs::create_dir_all(dotfile_path).context(format!("Can't create {}", DOTFILE_FOLDER))?;
    }

    let config_path = Path::new(CONFIG_FILE);

    // Initialize the configuration file
    if !config_path.exists() {
        let config = Config::default();

        let toml_content =
            toml::to_string_pretty(&config).context("Can't serialize the default configuration")?;

        fs::write(config_path, toml_content).context(format!("Can't create {}", CONFIG_FILE))?;
    }

    let mut gitignore_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(".gitignore")?;

    writeln!(gitignore_file, "{}", GITIGNORE_CONTENT)?;

    Ok(())
}

// Wipe all the files and folders of grayman
pub fn wipe() -> Result<()> {
    let dir_list = [Path::new(DOTFILE_FOLDER)];
    let file_list = [Path::new(CONFIG_FILE)];

    for path in dir_list {
        if path.exists() {
            remove_dir_all(path)?;
        }
    }

    for path in file_list {
        if path.exists() {
            remove_file(path)?;
        }
    }

    Ok(())
}
