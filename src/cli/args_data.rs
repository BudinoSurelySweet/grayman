use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author = "Marco Molossi", version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    /// Initialize Faber in the current directory.
    Init,

    Tui,

    /// Run the specified task. If no task was given it'll run the last runned task. If there is no task runned a prompt will appear.
    Forge(ExecutionData),

    Watch(ExecutionData),

    New {
        #[clap(subcommand)]
        action: NewAction,
    },

    Remove {
        #[clap(subcommand)]
        action: RemoveAction,
    },

    /// Clear all the content related to faber from the current directory.
    Clear,
}

#[derive(Args)]
pub struct ExecutionData {
    /// Name of the task to run.
    #[clap(short, long)]
    pub name: Option<String>,

    #[clap(short, long, conflicts_with = "name")]
    pub select: bool,
}

#[derive(Subcommand, Clone)]
pub enum NewAction {
    Var,
    Task,
}

#[derive(Subcommand, Clone)]
pub enum RemoveAction {
    Var,
    Task,
}
