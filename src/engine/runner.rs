use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_commands},
        extractor::get_tasks_to_execute,
    },
};
use anyhow::{Context, Result};

pub fn run_task_with_deps(task: &Task, config: &Config) -> Result<()> {
    let task_list = get_tasks_to_execute(task, config)?;

    for task in task_list {
        let command_list = create_commands(&task, config, StdioMode::Direct)?;

        for mut command in command_list {
            let child = command.spawn().context(format!(
                "An error occoured while spawning process \"{:?}\"",
                task.commands
            ))?;
            let output = child.wait_with_output()?;

            if !output.status.success() {
                let code = output.status.code().unwrap_or(-1);

                anyhow::bail!("Process failed with exit code \"{}\"", code);
            }
        }
    }

    Ok(())
}
