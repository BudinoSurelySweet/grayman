use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_commands},
        extractor::get_tasks_to_execute,
        multithread::stdxxx_handle::get_stdxxx_handles,
    },
};
use anyhow::Result;
use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

pub fn run_task_with_deps(task: Task, config: Config) -> Result<Receiver<String>> {
    let task_list = get_tasks_to_execute(&task, &config)?;
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        'first: for task in task_list {
            // Get the command of the current task
            let command_list = match create_commands(&task, &config, StdioMode::Piped) {
                Ok(list) => list,
                Err(error) => {
                    let _ = sender.send(format!("An error occoured: {}", error));
                    return;
                }
            };

            for mut command in command_list {
                // Spawn the process
                let mut child = match command.spawn() {
                    Ok(child) => child,
                    Err(error) => {
                        let _ = sender.send(format!("An error occoured: {}", error));
                        return;
                    }
                };

                // Spawn a thread for each stream to read stdout and stderr concurrently.
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                let (stdout_handle, stderr_handle) = get_stdxxx_handles(stdout, stderr, &sender);

                let _ = stdout_handle.join();
                let _ = stderr_handle.join();

                // Wait for the process to finish
                match child.wait() {
                    Ok(exit_code) => {
                        if !exit_code.success() {
                            break 'first;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(format!("Error while waiting for child: {}", error));
                    }
                }
            }
        }
    });

    Ok(receiver)
}
