use crate::{
    data::{Config, Task},
    executor::{
        checker::check_ciclic_dependencies,
        command_creator::{StdioMode, create_command},
    },
};
use anyhow::{Context, Result};
use std::collections::HashSet;

// Execute a task and the task it depends on recursively
pub fn execute_task_with_dependencies(
    task: (&str, &Task),
    config: &Config,
    executed: &mut HashSet<String>,
) -> Result<()> {
    if executed.is_empty() {
        let tasks_with_deps = config
            .tasks
            .iter()
            .map(|(name, task)| (name.clone(), task.depends_on.clone()))
            .collect();

        check_ciclic_dependencies(&tasks_with_deps)?;
    }

    if executed.contains(task.0) {
        return Ok(());
    }

    if let Some(dependencies) = &task.1.depends_on {
        for name in dependencies {
            let task = config
                .tasks
                .get(name)
                .context(format!("Task \"{}\" doesn't exists", task.0))?;

            execute_task_with_dependencies((name, &task), config, executed)?;
        }
    }

    let mut command = create_command(task, config, StdioMode::Direct)?;

    // Execute command
    let child = command.spawn().context(format!(
        "An error occoured while spawning process \"{}\"",
        task.1.command
    ))?;
    let output = child.wait_with_output()?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);

        anyhow::bail!("Process failed with exit code \"{}\"", code);
    }

    executed.insert(String::from(task.0));

    Ok(())
}
