use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author = "Marco Molossi", version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize Grayman in the current directory.
    Init,
    /// Launch the interactive terminal user interface (TUI).
    Tui,
    /// Run a task. Runs the last executed task if none is specified.
    Run(RunCommandData),
    /// Run a task and re-run it when files change. Runs the last executed task if none is specified.
    Watch(WatchCommandData),
    /// Add configuration entries (variables, tasks).
    Add {
        #[clap(subcommand)]
        action: AddAction,
    },
    /// Remove configuration entries (variables, tasks).
    Remove {
        #[clap(subcommand)]
        action: RemoveAction,
    },
    /// Remove all Grayman configuration and state from the current directory.
    Wipe,
}

#[derive(Args)]
pub struct RunCommandData {
    /// Name of the task to execute.
    #[clap(short, long)]
    pub name: Option<String>,
    /// Select a task interactively from a list.
    #[clap(short, long, conflicts_with = "name")]
    pub select: bool,
}

#[derive(Args)]
pub struct WatchCommandData {
    /// Name of the task to execute.
    #[clap(short, long)]
    pub name: Option<String>,
    /// Select a task interactively from a list.
    #[clap(short, long, conflicts_with = "name")]
    pub select: bool,
    /// Clear the console when restarting the task. ATTENTION: This action can't be undo.
    #[clap(short, long)]
    pub clear_on_restart: bool,
}

#[derive(Subcommand, Clone)]
pub enum AddAction {
    /// Add a new variable to the configuration.
    Var,
    /// Add a new task definition.
    Task,
}

#[derive(Subcommand, Clone)]
pub enum RemoveAction {
    /// Remove an existing variable from the configuration.
    Var,
    /// Remove a task definition.
    Task,
}
