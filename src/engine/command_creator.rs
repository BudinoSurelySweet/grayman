use crate::data::{Config, Task};
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub enum StdioMode {
    Direct,
    Piped,
}

pub fn create_commands(task: &Task, config: &Config, mode: StdioMode) -> Result<Vec<Command>> {
    let mut command_list = Vec::new();

    for current_command in &task.commands {
        let mut command;

        if let Some(shell) = task.shell
            && shell
        {
            command = Command::new("sh");
            command.arg("-c").arg(current_command);
        } else {
            let mut parts = current_command.split_whitespace();
            let program = parts
                .next()
                .context(format!("Command of \"{}\" can't be empty", task.name))?;

            command = Command::new(program);
            command.args(parts);
        }

        // Set the current working directory if the user specified it
        if let Some(cwd) = &task.cwd {
            command.current_dir(cwd);
        }

        // Inject the global environment variables
        if let Some(env) = config.env.clone() {
            command.envs(env);
        }

        // Inject the local environment variables
        if let Some(env) = &task.env {
            command.envs(env);
        }

        match mode {
            // Connect the IO streams to the parent's terminal
            StdioMode::Direct => command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),

            // Connect the IO streams to the parent
            StdioMode::Piped => command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        };

        command_list.push(command);
    }

    Ok(command_list)
}
