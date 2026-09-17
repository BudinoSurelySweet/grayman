use std::sync::mpsc;

use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_commands},
        extractor::get_tasks_to_execute,
        multithread::stdxxx_handle::get_stdxxx_handles,
    },
};
use anyhow::{Context, Result};

pub fn run_task_with_deps(task: &Task, config: &Config) -> Result<()> {
    let task_list = get_tasks_to_execute(task, config)?;

    for task in task_list {
        let command_list = create_commands(&task, config, StdioMode::Piped)?;

        for mut command in command_list {
            let mut child = command.spawn().context(format!(
                "An error occoured while spawning process \"{:?}\"",
                task.commands
            ))?;

            let (sender, receiver) = mpsc::channel();
            let (stdout_handle, stderr_handle) =
                get_stdxxx_handles(child.stdout.take(), child.stderr.take(), &sender);

            drop(sender);

            for line in receiver {
                println!("{}", line);
            }

            let status = child.wait()?;
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();

            if !status.success() {
                let code = status.code().unwrap_or(-1);

                anyhow::bail!("Process failed with exit code \"{}\"", code);
            }
        }
    }

    Ok(())
}
