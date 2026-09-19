use crate::{
    cli::{
        action_add::execute_add, action_init::execute_init, action_remove::execute_remove,
        action_run::execute_run, action_watch::execute_watch, action_wipe::execute_wipe,
        args_data::Cli,
    },
    tui::entry_point::run as tui_run,
};
use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args {
        _ if args.init => execute_init()?,
        _ if args.wipe => execute_wipe()?,
        _ if args.tui => tui_run()?,
        _ if args.add => execute_add()?,
        _ if args.remove => execute_remove()?,
        _ if args.select => {
            if !args.watch {
                execute_run(None, true)?;
            } else {
                execute_watch(None, true, args.clear_output)?;
            }
        }
        _ => {
            if !args.watch {
                execute_run(args.task_name, false)?;
            } else {
                execute_watch(args.task_name, false, args.clear_output)?;
            }
        }
    }

    Ok(())
}
