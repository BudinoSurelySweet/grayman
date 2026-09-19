use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[command(author = "Marco Molossi", version, about, group(
    ArgGroup::new("action")
        .required(false)
        .multiple(false)
        .args(["init", "tui", "add", "remove", "wipe"])
))]
pub struct Cli {
    /// Name of the task to execute.
    pub task_name: Option<String>,

    /// Select a task interactively from a list.
    #[clap(short, long, conflicts_with = "task_name")]
    pub select: bool,

    /// Select a task interactively from a list.
    #[clap(short, long)]
    pub watch: bool,

    #[clap(long, requires = "watch")]
    pub clear_output: bool,

    /// Initialize Grayman in the current directory.
    #[clap(short, long)]
    pub init: bool,

    /// Remove all Grayman configuration and state from the current directory.
    #[clap(long)]
    pub wipe: bool,

    /// Launch the interactive terminal user interface (TUI).
    #[clap(short, long)]
    pub tui: bool,

    /// Add configuration entries (variables, tasks).
    #[clap(long)]
    pub add: bool,

    /// Remove configuration entries (variables, tasks).
    #[clap(long)]
    pub remove: bool,
}
