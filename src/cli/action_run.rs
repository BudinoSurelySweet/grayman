use crate::{
    cli::args_data::RunCommandData,
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

pub fn execute_run(data: RunCommandData) -> Result<()> {
    let config = load_config()?;
    let task_name;

    if data.select {
        task_name = prompt_available_tasks(&config)?;
    } else if let Some(name) = data.name {
        task_name = name;
    } else if let Some(name) = Cache::load().last_task {
        task_name = name;
    } else {
        task_name = prompt_available_tasks(&config)?;
    }

    log!(info, "Task \"{}\" is selected", task_name);

    let task = config
        .tasks
        .iter()
        .find(|task| task.name == task_name)
        .context(format!("There is no task named \"{}\"", task_name))?;

    run_task_with_deps(task, &config)?;

    let cache = Cache {
        last_task: Some(task_name.clone()),
    };

    cache.save()?;

    log!(info, "Task \"{}\" exited with success", &task_name);

    Ok(())
}
