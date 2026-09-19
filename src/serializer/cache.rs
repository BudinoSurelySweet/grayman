use crate::serializer::data::{CACHE_FILE, DOTFILE_FOLDER};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    pub last_task: Option<String>,
}

impl Cache {
    pub fn load() -> Self {
        let path = PathBuf::from(format!("{}/{}", DOTFILE_FOLDER, CACHE_FILE));

        fs::read(path)
            .ok()
            .and_then(|bytes| postcard::from_bytes(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(format!("{}/{}", DOTFILE_FOLDER, CACHE_FILE));
        let bytes = postcard::to_allocvec(self)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)?;

        Ok(())
    }
}
