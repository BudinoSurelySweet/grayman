use crate::{
    cli::args_data::ExecutionData,
    data::Config,
    engine::watcher::{EventMessageType, WatcherEventData, start_watcher},
    log,
    serializer::{
        config::load_config,
        last_task::{get_last_task_name, save_last_task_name},
    },
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

pub fn execute_watch(data: ExecutionData) -> Result<()> {
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

    let Some(task) = config.tasks.iter().find(|task| task.name == task_name) else {
        return Err(anyhow!(format!("There is no task named \"{}\"", task_name)));
    };

    let emit_message = |data: WatcherEventData| {
        match data.message_type {
            EventMessageType::Info => log!(info, "{}", data.message),
            EventMessageType::_Warn => log!(warn, "{}", data.message),
            EventMessageType::_Error => log!(error, "{}", data.message),
        }

        Ok(())
    };

    start_watcher(&task, &config, emit_message)?;

    save_last_task_name(&task_name)?;

    log!(info, "Task \"{}\" exited with success", &task_name);

    Ok(())
}
