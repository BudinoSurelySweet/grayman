use crate::{
    cli::{
        action_add::execute_add,
        action_init::execute_init,
        action_remove::execute_remove,
        action_run::execute_run,
        action_watch::execute_watch,
        action_wipe::execute_wipe,
        args_data::{Action, Cli},
    },
    tui::entry_point::run as tui_run,
};
use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.action {
        Action::Init => execute_init()?,
        Action::Tui => tui_run()?,
        Action::Run(data) => execute_run(data)?,
        Action::Watch(data) => execute_watch(data)?,
        Action::Add { action } => execute_add(action)?,
        Action::Remove { action } => execute_remove(action)?,
        Action::Wipe => execute_wipe()?,
    }

    Ok(())
}
