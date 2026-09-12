use crate::{
    cli::args_data::ExecutionData,
    data::Config,
    executor::runner::execute_task_with_dependencies,
    info,
    serializer::{
        config::load_config,
        last_task::{get_last_task_name, save_last_task_name},
    },
};
use anyhow::{Context, Result};
use std::collections::HashSet;

fn prompt_available_tasks(config: &Config) -> Result<String> {
    let options = config.tasks.keys().cloned().collect();
    let task_name = inquire::Select::new("What task do you want to forge?", options).prompt()?;

    Ok(task_name)
}

pub fn execute_forge(data: ExecutionData) -> Result<()> {
    let config = load_config()?;
    let task_name;

    if data.select {
        task_name = prompt_available_tasks(&config)?;
    } else if let Some(name) = data.name {
        task_name = name;
    } else if let Ok(name) = get_last_task_name() {
        task_name = name;
    } else {
        task_name = prompt_available_tasks(&config)?;
    }

    let task = config
        .tasks
        .get(&task_name)
        .context(format!("There is no task named \"{}\"", task_name))?;

    let mut executed_task = HashSet::new();

    execute_task_with_dependencies((&task_name, task), &config, &mut executed_task)?;

    save_last_task_name(&task_name)?;

    info!("Task \"{}\" exited with success", &task_name);

    Ok(())
}
