use crate::{
    data::Config,
    engine::watcher::start_watcher,
    log,
    serializer::{cache::Cache, config::load_config},
};
use anyhow::{Result, anyhow};

fn prompt_available_tasks(config: &Config) -> Result<String> {
    let options = config
        .tasks
        .iter()
        .filter_map(|task| {
            if task.watch.is_some() {
                Some(task.name.clone())
            } else {
                None
            }
        })
        .collect();
    let task_name = inquire::Select::new("What task do you want to watch?", options).prompt()?;

    Ok(task_name)
}

pub fn execute_watch(
    task_name: Option<String>,
    force_select: bool,
    clear_output: bool,
) -> Result<()> {
    let config = load_config()?;

    let task_name = if !force_select {
        task_name.or_else(|| Cache::load().last_task)
    } else {
        None
    };

    let task_name = match task_name {
        Some(name) => name,
        None => prompt_available_tasks(&config)?,
    };

    let Some(task) = config.tasks.iter().find(|task| task.name == task_name) else {
        return Err(anyhow!(format!("There is no task named \"{}\"", task_name)));
    };

    let cache = Cache {
        last_task: Some(task_name.clone()),
    };

    cache.save()?;

    start_watcher(task, &config, clear_output)?;

    log!(info, "Task \"{}\" exited with success", &task_name);

    Ok(())
}
