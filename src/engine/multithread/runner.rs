use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_command},
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
        for task in task_list {
            // Get the command of the current task
            let mut command = match create_command(&task, &config, StdioMode::Piped) {
                Ok(command) => command,
                Err(error) => {
                    let _ = sender.send(format!("An error occoured: {}", error));
                    return;
                }
            };

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
                    let _ = sender.send(format!("Process ended with exit code {}", exit_code));
                }
                Err(error) => {
                    let _ = sender.send(format!("Error while waiting for child: {}", error));
                }
            }
        }
    });

    Ok(receiver)
}
