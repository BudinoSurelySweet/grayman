use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_command},
        extractor::get_tasks_to_execute,
    },
};
use anyhow::{Result, anyhow};
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, RecursiveMode},
};
use std::{path::Path, process::Child, sync::mpsc, time::Duration};

#[derive(Debug)]
pub enum EventMessageType {
    Info,
    _Warn,
    _Error,
}

#[derive(Debug)]
pub struct WatcherEventData {
    pub message: String,
    pub message_type: EventMessageType,
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

    // Register the watch_list into the debouncer
    for path in watch_list {
        if let Err(e) = debouncer.watch(Path::new(&path), RecursiveMode::Recursive) {
            return Err(anyhow!(format!("Can't monitor \"{}\": {}", path, e)));
        }
    }

    let mut current_child: Option<Child> = None;

    let task_list = get_tasks_to_execute(task, config)?;

    let mut execute = move |message: Option<&str>| -> Result<()> {
        if let Some(mut child) = current_child.take() {
            let _ = child.kill(); // Kill the child
            let _ = child.wait(); // Clean the process
        }

        if let Some(message) = message {
            emit_message(WatcherEventData {
                message: String::from(message),
                message_type: EventMessageType::Info,
            })?;
        }

        for task in &task_list {
            let mut command = create_command(task, config, StdioMode::Direct)?;

            current_child = Some(command.spawn()?);
        }

        Ok(())
    };

    execute(None)?;

    for event in receiver {
        match event {
            Err(err) => return Err(anyhow!("Error while watching: {:?}", err)),
            Ok(event_list) => {
                for event in event_list {
                    match event.event.kind {
                        EventKind::Any => {}
                        EventKind::Access(_) => {}
                        EventKind::Create(_) => execute(Some("File creation detected"))?,
                        EventKind::Modify(_) => execute(Some("File modification detected"))?,
                        EventKind::Remove(_) => execute(Some("File remove detected"))?,
                        EventKind::Other => {}
                    }
                }
            }
        }
    }

    Ok(())
}
