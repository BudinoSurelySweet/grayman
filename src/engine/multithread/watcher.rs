use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_command},
        extractor::get_tasks_to_execute,
        multithread::stdxxx_handle::get_stdxxx_handles,
    },
};
use anyhow::{Result, anyhow};
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, RecursiveMode},
};
use std::{
    path::Path,
    process::Child,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

pub fn start_watcher(task: Task, config: Config) -> Result<Receiver<String>> {
    let Some(watch_list) = task.clone().watch.filter(|w| !w.is_empty()) else {
        return Err(anyhow!("No available watch list for the specified task"));
    };

    let (debouncer_sender, debouncer_receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(200), None, debouncer_sender)?;

    // Register the watch_list into the debouncer
    for path in watch_list {
        if let Err(e) = debouncer.watch(Path::new(&path), RecursiveMode::Recursive) {
            return Err(anyhow!(format!("Can't monitor \"{}\": {}", path, e)));
        }
    }

    let mut child_list: Vec<Child> = Vec::new();
    let task_list = get_tasks_to_execute(&task, &config)?;
    let (sender, receiver) = mpsc::channel();

    let mut execute = {
        let sender = sender.clone();

        move |message: Option<&str>| {
            for mut child in child_list.drain(..) {
                let _ = child.kill(); // Kill the child
                let _ = child.wait(); // Clean the process
            }

            if let Some(message) = message {
                let _ = sender.send(String::from(message));
            }

            for task in &task_list {
                let mut command = match create_command(task, &config, StdioMode::Piped) {
                    Ok(command) => command,
                    Err(error) => {
                        let _ = sender.send(format!("{}", error));
                        return;
                    }
                };

                let child = match command.spawn() {
                    Err(error) => {
                        let _ = sender.send(format!("{}", error));
                        return;
                    }
                    Ok(mut child) => {
                        let stdout = child.stdout.take();
                        let stderr = child.stderr.take();

                        let (_, _) = get_stdxxx_handles(stdout, stderr, &sender);

                        child
                    }
                };

                child_list.push(child);
            }
        }
    };

    thread::spawn(move || {
        // By moving the debouncer in this local variable we keep
        // it's life cycle bounded to this thread. Once the thread
        // end the debouncer will be destroyed.
        let _keepalive_debouncer = debouncer;

        // First execution of the task
        execute(None);

        for event in debouncer_receiver {
            match event {
                Err(err) => {
                    let _ = sender.send(format!("Error while watching: {:?}", err));
                    return;
                }
                Ok(event_list) => {
                    for event in event_list {
                        match event.event.kind {
                            EventKind::Any => {}
                            EventKind::Access(_) => {}
                            EventKind::Create(_) => execute(Some("File creation detected")),
                            EventKind::Modify(_) => execute(Some("File modification detected")),
                            EventKind::Remove(_) => execute(Some("File remove detected")),
                            EventKind::Other => {}
                        }
                    }
                }
            }
        }
    });

    Ok(receiver)
}
