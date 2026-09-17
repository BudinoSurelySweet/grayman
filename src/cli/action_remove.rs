use crate::{
    cli::args_data::RemoveAction,
    data::Task,
    log,
    serializer::config::{load_config, save_config},
};
use anyhow::{Result, anyhow};

// TODO: Devo spostare la logica di rimozione nell'engine
pub fn execute_remove(action: RemoveAction) -> Result<()> {
    let mut config = load_config()?;

    match action {
        RemoveAction::Var => {
            let Some(mut env) = config.env else {
                return Err(anyhow!("There are no environment variables"));
            };
            let options = env.iter().map(|(name, _)| name.clone()).collect();
            let vars_to_remove =
                inquire::MultiSelect::new("What variables do you want to remove?", options)
                    .prompt()?;

            for v in &vars_to_remove {
                env.remove(v);
            }

            config.env = Some(env);

            save_config(config)?;

            if vars_to_remove.is_empty() {
                log!(info, "No variable was removed");
            } else if vars_to_remove.len() == 1 {
                log!(
                    info,
                    "Variable \"{}\" removed succesfully",
                    vars_to_remove[0]
                );
            } else {
                log!(info, "Variables {:?} removed succesfully", vars_to_remove);
            }
        }
        RemoveAction::Task => {
            let options = config.tasks.iter().map(|task| task.name.clone()).collect();
            let tasks_to_remove =
                inquire::MultiSelect::new("What tasks do you want to remove?", options).prompt()?;
            let tasks: Vec<Task> = config
                .tasks
                .iter()
                .filter_map(|task| {
                    if tasks_to_remove.contains(&task.name) {
                        Some(task.clone())
                    } else {
                        None
                    }
                })
                .collect();

            config.tasks = tasks;

            save_config(config)?;

            if tasks_to_remove.is_empty() {
                log!(info, "No task was removed");
            } else if tasks_to_remove.len() == 1 {
                log!(info, "Task \"{}\" removed succesfully", tasks_to_remove[0]);
            } else {
                log!(info, "Tasks {:?} removed succesfully", tasks_to_remove);
            }
        }
    }

    Ok(())
}
