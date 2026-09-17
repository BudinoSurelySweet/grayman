use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Config file structure
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub env: Option<HashMap<String, String>>,
    pub tasks: Vec<Task>,
}

// Task specific structure
#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub name: String,
    pub commands: Vec<String>,
    pub description: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub shell: Option<bool>,
    pub watch: Option<Vec<String>>,
}
