use crate::{
    data::{Config, Task},
    engine::checker::check_ciclic_dependencies,
};
use anyhow::{Context, Result};
use std::collections::HashSet;

fn get_tasks_to_execute_recursively(
    task: &Task,
    config: &Config,
    executed: &mut HashSet<String>,
) -> Result<Vec<Task>> {
    if executed.is_empty() {
        let tasks_with_deps = config
            .tasks
            .iter()
            .map(|task| (task.name.clone(), task.depends_on.clone()))
            .collect();

        check_ciclic_dependencies(&tasks_with_deps)?;
    }

    let mut task_list = Vec::new();

    if executed.contains(&task.name) {
        return Ok(task_list);
    }

    if let Some(dependencies) = &task.depends_on {
        for name in dependencies {
            let task = config
                .tasks
                .iter()
                .find(|task| task.name == *name)
                .context(format!("Task \"{}\" doesn't exists", task.name))?;

            let mut dependencies = get_tasks_to_execute_recursively(task, config, executed)?;

            task_list.append(&mut dependencies);
        }
    }

    task_list.push(task.clone());
    executed.insert(String::from(task.name.clone()));

    Ok(task_list)
}

pub fn get_tasks_to_execute(task: &Task, config: &Config) -> Result<Vec<Task>> {
    let mut executed_tasks = HashSet::new();

    get_tasks_to_execute_recursively(task, config, &mut executed_tasks)
}
