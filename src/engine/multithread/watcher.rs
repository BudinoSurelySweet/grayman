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

    let mut child: Option<Child> = None;
    let task_list = get_tasks_to_execute(&task, &config)?;
    let (sender, receiver) = mpsc::channel();

    let mut execute = {
        let sender = sender.clone();

        move || {
            if let Some(mut child) = child.take() {
                let _ = child.kill(); // Kill the child
                let _ = child.wait(); // Clean the process
            }

            for (i, task) in task_list.iter().enumerate() {
                let mut command = match create_command(task, &config, StdioMode::Piped) {
                    Ok(command) => command,
                    Err(error) => {
                        let _ = sender.send(format!("{}", error));
                        return;
                    }
                };

                let is_last_element = i == task_list.len() - 1;

                if is_last_element {
                    match command.spawn() {
                        Err(error) => {
                            let _ = sender.send(format!("{}", error));
                            return;
                        }
                        Ok(mut new_child) => {
                            let stdout = new_child.stdout.take();
                            let stderr = new_child.stderr.take();

                            let (_, _) = get_stdxxx_handles(stdout, stderr, &sender);

                            child = Some(new_child);
                        }
                    };
                } else {
                    match command.spawn() {
                        Err(error) => {
                            let _ = sender.send(format!("Spawn error: {}", error));
                            return;
                        }
                        Ok(mut temp_child) => {
                            let stdout = temp_child.stdout.take();
                            let stderr = temp_child.stderr.take();

                            let (_, _) = get_stdxxx_handles(stdout, stderr, &sender);

                            match temp_child.wait() {
                                Ok(status) if !status.success() => {
                                    let _ = sender.send(format!("{}", status));
                                    break;
                                }
                                Err(error) => {
                                    let _ = sender.send(format!("Wait error: {}", error));
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    };

    thread::spawn(move || {
        // By moving the debouncer in this local variable we keep
        // it's life cycle bounded to this thread. Once the thread
        // end the debouncer will be destroyed.
        let _keepalive_debouncer = debouncer;

        // First execution of the task
        execute();

        for event in debouncer_receiver {
            match event {
                Err(err) => {
                    let _ = sender.send(format!("Error while watching: {:?}", err));
                    return;
                }
                Ok(event_list) => {
                    for event in event_list {
                        match event.event.kind {
                            EventKind::Any | EventKind::Access(_) | EventKind::Other => {}
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                                execute()
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(receiver)
}
