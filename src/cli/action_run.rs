use crate::{
    data::Config,
    engine::runner::run_task_with_deps,
    log,
    serializer::{cache::Cache, config::load_config},
};
use anyhow::{Context, Result};

fn prompt_available_tasks(config: &Config) -> Result<String> {
    let options = config.tasks.iter().map(|task| task.name.clone()).collect();
    let task_name = inquire::Select::new("What task do you want to run?", options).prompt()?;

    Ok(task_name)
}

pub fn execute_run(task_name: Option<String>, force_select: bool) -> Result<()> {
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

    let task = config
        .tasks
        .iter()
        .find(|task| task.name == task_name)
        .context(format!("There is no task named \"{}\"", task_name))?;

    log!(info, "Task \"{}\" is selected", task_name);

    run_task_with_deps(task, &config)?;

    let cache = Cache {
        last_task: Some(task_name.clone()),
    };

    cache.save()?;

    log!(info, "Task \"{}\" exited with success", &task_name);

    Ok(())
}
