use crate::{
    cli::{
        action_add::execute_add,
        action_init::execute_init,
        action_remove::execute_remove,
        action_run::execute_run,
        action_watch::execute_watch,
        action_wipe::execute_wipe,
        args_data::{Cli, Command},
    },
    tui::entry_point::run as tui_run,
};
use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Init => execute_init()?,
        Command::Tui => tui_run()?,
        Command::Run(data) => execute_run(data)?,
        Command::Watch(data) => execute_watch(data)?,
        Command::Add { action } => execute_add(action)?,
        Command::Remove { action } => execute_remove(action)?,
        Command::Wipe => execute_wipe()?,
    }

    Ok(())
}
