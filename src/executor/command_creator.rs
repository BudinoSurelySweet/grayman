use crate::data::{Config, Task};
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub enum StdioMode {
    Direct,
    Piped,
}

pub fn create_command(task: (&str, &Task), config: &Config, mode: StdioMode) -> Result<Command> {
    let mut command;

    if let Some(shell) = task.1.shell
        && shell
    {
        command = Command::new("sh");
        command.arg("-c").arg(&task.1.command);
    } else {
        let mut parts = task.1.command.split_whitespace();
        let program = parts
            .next()
            .context(format!("Command of \"{}\" can't be empty", task.0))?;

        command = Command::new(program);
        command.args(parts);
    }

    // Set the current working directory if the user specified it
    if let Some(cwd) = &task.1.cwd {
        command.current_dir(cwd);
    }

    // Inject the global environment variables
    if let Some(env) = config.env.clone() {
        command.envs(env);
    }

    // Inject the local environment variables
    if let Some(env) = &task.1.env {
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

    Ok(command)
}
