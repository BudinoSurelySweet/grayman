// use crate::serializer::data::DOTFILE_FOLDER;
// use anyhow::{Context, Result};
// use std::{fs, path::PathBuf};

// pub fn get_last_task_name() -> Result<String> {
//     let path = PathBuf::from(format!("{}/{}", DOTFILE_FOLDER, LAST_TASK_FILE));
//     let task_name = fs::read_to_string(&path)?;

//     Ok(task_name)
// }

// pub fn save_last_task_name(task_name: &str) -> Result<()> {
//     let path = PathBuf::from(DOTFILE_FOLDER);

//     if !path.exists() {
//         fs::create_dir_all(&path).context(format!("Can't create folder {}", DOTFILE_FOLDER))?;
//     }

//     let path = path.join(LAST_TASK_FILE);

//     fs::write(&path, task_name).context("Can't save last task to file")?;

//     Ok(())
// }
