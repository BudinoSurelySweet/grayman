use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_command},
        runner::run_task_with_deps,
    },
};
use anyhow::{Result, anyhow};
use notify_debouncer_full::{
    Debouncer, NoCache, new_debouncer,
    notify::{EventKind, INotifyWatcher, RecursiveMode},
};
use std::{path::Path, process::Child, sync::mpsc, time::Duration};

#[derive(Debug)]
pub enum EventMessageType {
    Info,
    Warn,
    Error,
}

#[derive(Debug)]
pub struct WatcherEventData {
    pub message: String,
    pub message_type: EventMessageType,
}

fn register_list_into_watcher(
    debouncer: &mut Debouncer<INotifyWatcher, NoCache>,
    watch_list: Vec<String>,
) -> Result<()> {
    for path in watch_list {
        if let Err(e) = debouncer.watch(Path::new(&path), RecursiveMode::Recursive) {
            return Err(anyhow!(format!("Can't monitor \"{}\": {}", path, e)));
        }
    }

    Ok(())
}

fn execute_command<F>(task: &Task, config: &Config, emit_message: F) -> Result<Option<Child>>
where
    F: Fn(WatcherEventData) -> Result<()>,
{
    if let Some(dependencies) = &task.depends_on {
        for name in dependencies {
            let Some(task) = config.tasks.iter().find(|task| task.name == *name) else {
                return Err(anyhow!("Task doesn't exists"));
            };

            if let Err(e) = run_task_with_deps(task, config) {
                emit_message(WatcherEventData {
                    message: format!("Error captured\n\n{:?}", e),
                    message_type: EventMessageType::Error,
                })?;

                return Ok(None);
            }
        }
    }

    let mut command = create_command(task, config, StdioMode::Direct)?;
    let child = Some(command.spawn()?);

    Ok(child)
}

pub fn start_watcher<F>(task: &Task, config: &Config, emit_message: F) -> Result<()>
where
    F: Fn(WatcherEventData) -> Result<()>,
{
    let Some(watch_list) = task.clone().watch.filter(|w| !w.is_empty()) else {
        return Err(anyhow!("No available watch list for the specified task"));
    };

    let (sender, receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(200), None, sender)?;

    register_list_into_watcher(&mut debouncer, watch_list)?;

    let mut current_child = execute_command(task, config, &emit_message)?;

    let mut restart = move |message| -> Result<()> {
        if let Some(mut child) = current_child.take() {
            let _ = child.kill(); // Kill the child
            let _ = child.wait(); // Clean the process
        }

        emit_message(WatcherEventData {
            message: String::from(message),
            message_type: EventMessageType::Info,
        })?;

        current_child = execute_command(task, config, &emit_message)?;

        Ok(())
    };

    for event in receiver {
        match event {
            Err(err) => return Err(anyhow!("Error while watching: {:?}", err)),
            Ok(event_list) => {
                for event in event_list {
                    match event.event.kind {
                        EventKind::Any => {}
                        EventKind::Access(_) => {}
                        EventKind::Create(_) => restart("File creation detected")?,
                        EventKind::Modify(_) => restart("File modification detected")?,
                        EventKind::Remove(_) => restart("File remove detected")?,
                        EventKind::Other => {}
                    }
                }
            }
        }
    }

    Ok(())
}
