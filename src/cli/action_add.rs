use crate::{
    cli::args_data::AddAction,
    data::Task,
    log,
    serializer::config::{load_config, save_config},
};
use anyhow::Result;
use inquire::validator::{MinLengthValidator, Validation};
use std::collections::HashMap;

fn prompt_task_creation() -> Result<Task> {
    let config = load_config()?;
    let task_list: Vec<String> = config.tasks.iter().map(|task| task.name.clone()).collect();

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

    let mut commands: Vec<String> = Vec::new();

    let command = inquire::Text::new("What command do you want to execute?")
        .with_validator(MinLengthValidator::new(1))
        .prompt()?;

    commands.push(command);

    loop {
        let user_want_to_continue = inquire::Confirm::new("Do you want to add another command?")
            .with_default(false)
            .with_starting_input("No")
            .with_help_message("y/N")
            .prompt()?;

        if !user_want_to_continue {
            break;
        }

        let command = inquire::Text::new("What command do you want to add?")
            .with_validator(MinLengthValidator::new(1))
            .prompt()?;

        commands.push(command);
    }

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
    let shell = if shell { None } else { Some(shell) };
    let watch = if watch.is_empty() { None } else { Some(watch) };

    let task = Task {
        name,
        commands,
        description,
        depends_on,
        env,
        cwd,
        shell,
        watch,
    };

    Ok(task)
}

// TODO: Devo spostare la logica di creazione nell'engine
pub fn execute_add(action: AddAction) -> Result<()> {
    let mut config = load_config()?;

    match action {
        AddAction::Var => {
            let mut env = config.env.unwrap_or_else(HashMap::new);

            let var_name = inquire::Text::new("What's the variable's name?").prompt()?;
            let var_value = inquire::Text::new(&format!("{} =", var_name)).prompt()?;

            env.insert(var_name.clone(), var_value);
            config.env = Some(env);
            save_config(config)?;

            log!(
                info,
                "Environment variable \"{}\" created succesfully",
                var_name
            );
        }
        AddAction::Task => {
            let task = prompt_task_creation()?;
            let task_name = task.name.clone();

            config.tasks.push(task);
            save_config(config)?;

            log!(info, "Task \"{}\" created succesfully", task_name);
        }
    }

    Ok(())
}
