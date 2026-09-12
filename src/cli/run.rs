use crate::cli::{
    action_clear::execute_clear,
    action_forge::execute_forge,
    action_init::execute_init,
    action_new::execute_new,
    action_remove::execute_remove,
    action_watch::execute_watch,
    args_data::{Action, Cli},
};
use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.action {
        Action::Init => execute_init()?,
        Action::Tui => todo!("Implement the tui"),
        Action::Forge(data) => execute_forge(data)?,
        Action::Watch(data) => execute_watch(data)?,
        Action::New { action } => execute_new(action)?,
        Action::Remove { action } => execute_remove(action)?,
        Action::Clear => execute_clear()?,
    }

    Ok(())
}
