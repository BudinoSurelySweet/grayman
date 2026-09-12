use crate::data::{Config, Task};
use crate::executor::runner::execute_task_with_dependencies;
use crate::executor::watcher::{EventMessageType, WatcherEventData, start_watcher};
use crate::serializer::config::{load_config, save_config};
use crate::serializer::essential::{init, wipe};
use crate::serializer::last_task::{get_last_task_name, save_last_task_name};
use crate::{error, info, warn};
use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use inquire::validator::{MinLengthValidator, Validation};
use std::collections::{HashMap, HashSet};

#[derive(Parser)]
#[command(author = "Marco Molossi", version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,
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

fn execute_init() -> Result<()> {
    let result = init(false);

    match result {
        Ok(_) => info!("You succesfully initialized the configuration"),
        Err(err) => {
            error!("{}\n", err);

            let user_wants_to_reinit = inquire::Confirm::new("Do you want to re-initialize?")
                .with_help_message("y/N")
                .with_starting_input("No")
                .prompt()?;

            if user_wants_to_reinit {
                init(true)?;
                info!("\nYou succesfully re-initialized the configuration");
            }
        }
    }

    Ok(())
}

fn prompt_available_tasks(config: &Config) -> Result<String> {
    let options = config.tasks.keys().cloned().collect();
    let task_name = inquire::Select::new("What task do you want to forge?", options).prompt()?;

    Ok(task_name)
}

fn execute_forge(data: ExecutionData) -> Result<()> {
    let config = load_config()?;
    let task_name;

    if data.select {
        task_name = prompt_available_tasks(&config)?;
    } else if let Some(name) = data.name {
        task_name = name;
    } else if let Ok(name) = get_last_task_name() {
        task_name = name;
    } else {
        task_name = prompt_available_tasks(&config)?;
    }

    let task = config
        .tasks
        .get(&task_name)
        .context(format!("There is no task named \"{}\"", task_name))?;

    let mut executed_task = HashSet::new();

    execute_task_with_dependencies((&task_name, task), &config, &mut executed_task)?;

    save_last_task_name(&task_name)?;

    info!("Task \"{}\" exited with success", &task_name);

    Ok(())
}

fn execute_watch(data: ExecutionData) -> Result<()> {
    let config = load_config()?;
    let task_name;

    if data.select {
        task_name = prompt_available_tasks(&config)?;
    } else if let Some(name) = data.name {
        task_name = name;
    } else if let Ok(name) = get_last_task_name() {
        task_name = name;
    } else {
        task_name = prompt_available_tasks(&config)?;
    }

    let Some(task) = config.tasks.get(&task_name) else {
        return Err(anyhow!("Task doesn't exists"));
    };

    let emit_message = |data: WatcherEventData| {
        match data.message_type {
            EventMessageType::Info => info!("{}", data.message),
            EventMessageType::Warn => warn!("{}", data.message),
            EventMessageType::Error => error!("{}", data.message),
        }

        Ok(())
    };

    start_watcher((&task_name, task), &config, emit_message)?;

    save_last_task_name(&task_name)?;

    info!("Task \"{}\" exited with success", &task_name);

    Ok(())
}

fn prompt_task_creation() -> Result<(String, Task)> {
    let config = load_config()?;
    let task_list: Vec<String> = config.tasks.keys().cloned().collect();

    let name_validator = |input: &str| {
        let input = input.trim();

        if input.is_empty() {
            return Ok(Validation::Invalid("Name can't be empty.".into()));
        }

        if task_list.iter().any(|task| task == input) {
            return Ok(Validation::Invalid(
                "There is already a task with the same name.".into(),
            ));
        }

        Ok(Validation::Valid)
    };

    let name = inquire::Text::new("What's the task's name?")
        .with_validator(name_validator)
        .prompt()?;
    let cwd = inquire::Text::new("In which directory will be executed?")
        .with_help_message("empty = current")
        .prompt()?;
    let command = inquire::Text::new("What command do you want to execute?")
        .with_validator(MinLengthValidator::new(1))
        .prompt()?;
    let shell = inquire::Confirm::new("Enable execution inside shell?")
        .with_default(true)
        .with_starting_input("Yes")
        .with_help_message("Y/n")
        .prompt()?;
    let description = inquire::Text::new("Insert a description:")
        .with_help_message("You can leave this empty")
        .prompt()?;
    let mut depends_on: Vec<String> = Vec::new();

    loop {
        let user_want_to_continue = inquire::Confirm::new("Do you want to add a dependency?")
            .with_default(false)
            .with_starting_input("No")
            .with_help_message("y/N")
            .prompt()?;

        if !user_want_to_continue {
            break;
        }

        let task = inquire::Select::new(
            "What task do you want to add as a dependency?",
            task_list.clone(),
        )
        .prompt()?;

        depends_on.push(task);
    }

    let mut env: HashMap<String, String> = HashMap::new();

    loop {
        let user_want_to_continue =
            inquire::Confirm::new("Do you want to add a local environment variable?")
                .with_default(false)
                .with_starting_input("No")
                .with_help_message("y/N")
                .prompt()?;

        if !user_want_to_continue {
            break;
        }

        let key = inquire::Text::new("Variable name:")
            .with_validator(MinLengthValidator::new(1))
            .prompt()?;
        let value = inquire::Text::new(&format!("{} =", key))
            .with_validator(MinLengthValidator::new(1))
            .prompt()?;

        env.insert(key, value);
    }

    let mut watch: Vec<String> = Vec::new();

    loop {
        let user_want_to_continue =
            inquire::Confirm::new("Do you want to add something to the watcher?")
                .with_default(false)
                .with_starting_input("No")
                .with_help_message("y/N")
                .prompt()?;

        if !user_want_to_continue {
            break;
        }

        let path = inquire::Text::new("What do you want to watch?").prompt()?;

        watch.push(path);
    }

    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };
    let depends_on = if depends_on.is_empty() {
        None
    } else {
        Some(depends_on)
    };
    let env = if env.is_empty() { None } else { Some(env) };
    let cwd = if cwd.is_empty() { None } else { Some(cwd) };
    let shell = Some(shell);
    let watch = if watch.is_empty() { None } else { Some(watch) };

    let task = Task {
        command,
        description,
        depends_on,
        env,
        cwd,
        shell,
        watch,
    };

    Ok((name, task))
}

// TODO: Devo spostare la logica di creazione nell'engine
fn execute_new(action: NewAction) -> Result<()> {
    let mut config = load_config()?;

    match action {
        NewAction::Var => {
            let mut env = config.env.unwrap_or_else(|| HashMap::new());

            let var_name = inquire::Text::new("What's the variable's name?").prompt()?;
            let var_value = inquire::Text::new(&format!("{} =", var_name)).prompt()?;

            env.insert(var_name.clone(), var_value);
            config.env = Some(env);
            save_config(&config)?;

            info!("Environment variable \"{}\" created succesfully", var_name);
        }
        NewAction::Task => {
            let (name, task) = prompt_task_creation()?;

            config.tasks.insert(name.clone(), task);
            save_config(&config)?;

            info!("Task \"{}\" created succesfully", name);
        }
    }

    Ok(())
}

// TODO: Devo spostare la logica di rimozione nell'engine
fn execute_remove(action: RemoveAction) -> Result<()> {
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

            save_config(&config)?;

            if vars_to_remove.is_empty() {
                info!("No variable was removed");
            } else if vars_to_remove.len() == 1 {
                info!("Variable \"{}\" removed succesfully", vars_to_remove[0]);
            } else {
                info!("Variables {:?} removed succesfully", vars_to_remove);
            }
        }
        RemoveAction::Task => {
            let options = config.tasks.iter().map(|(name, _)| name.clone()).collect();
            let tasks_to_remove =
                inquire::MultiSelect::new("What tasks do you want to remove?", options).prompt()?;

            for t in &tasks_to_remove {
                config.tasks.remove(t);
            }

            save_config(&config)?;

            if tasks_to_remove.is_empty() {
                info!("No task was removed");
            } else if tasks_to_remove.len() == 1 {
                info!("Task \"{}\" removed succesfully", tasks_to_remove[0]);
            } else {
                info!("Tasks {:?} removed succesfully", tasks_to_remove);
            }
        }
    }

    Ok(())
}

fn execute_clear() -> Result<()> {
    let user_want_to_delete = inquire::Confirm::new("Do you want to delete all faber's data?")
        .with_help_message("y/N")
        .with_starting_input("No")
        .with_default(false)
        .prompt()?;

    if user_want_to_delete {
        wipe()?;
    }

    Ok(())
}

pub fn run() -> Result<()> {
    let args = Cli::parse();

    match args.action {
        None => todo!("Implement the tui"),
        Some(Action::Init) => execute_init()?,
        Some(Action::Tui) => todo!("Implement the tui"),
        Some(Action::Forge(data)) => execute_forge(data)?,
        Some(Action::Watch(data)) => execute_watch(data)?,
        Some(Action::New { action }) => execute_new(action)?,
        Some(Action::Remove { action }) => execute_remove(action)?,
        Some(Action::Clear) => execute_clear()?,
    }

    Ok(())
}
