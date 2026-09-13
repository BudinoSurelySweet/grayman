use anyhow::{Ok, Result, anyhow};
use std::collections::{HashMap, HashSet};

fn check_branch(
    branch: &mut Vec<String>,
    visited: &mut HashSet<String>,
    task: &String,
    task_list: &HashMap<String, Option<Vec<String>>>,
) -> Result<()> {
    if visited.contains(task) {
        return Ok(());
    }

    if branch.is_empty() {
        branch.push(task.clone());
    }

    let Some(Some(dependencies)) = task_list.get(task) else {
        visited.insert(task.clone());
        branch.pop();

        return Ok(());
    };

    for d in dependencies {
        if branch.contains(d) {
            return Err(anyhow!(format!(
                "Recursive dependency found: {} -> [{}]",
                branch.join(" -> "),
                d
            )));
        }

        branch.push(d.clone());

        check_branch(branch, visited, d, task_list)?;
    }

    visited.insert(task.clone());
    branch.pop();

    Ok(())
}

pub fn check_ciclic_dependencies(tasks: &HashMap<String, Option<Vec<String>>>) -> Result<()> {
    let mut visited = HashSet::new();

    for (name, _) in tasks {
        let mut branch = Vec::new();

        check_branch(&mut branch, &mut visited, &name, tasks)?;
    }

    Ok(())
}
