use crate::{
    data::{Config, Task},
    engine::{
        command_creator::{StdioMode, create_commands},
        extractor::get_tasks_to_execute,
        multithread::stdxxx_handle::get_stdxxx_handles,
    },
    log,
};
use anyhow::{Result, anyhow};
use crossterm::{cursor, execute, terminal};
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, RecursiveMode},
};
use std::{path::Path, process::Child, sync::mpsc, time::Duration};

fn clear_terminal() -> Result<()> {
    execute!(
        std::io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    Ok(())
}

fn execute(task_list: &[Task], config: &Config, child: &mut Option<Child>) -> Result<()> {
    if let Some(mut child) = child.take() {
        let _ = child.kill(); // Kill the child
        let _ = child.wait(); // Clean the process
    }

    'outer: for (i, task) in task_list.iter().enumerate() {
        let command_list = create_commands(task, config, StdioMode::Piped)?;

        for mut command in command_list {
            let mut c = command.spawn()?;

            let (sender, receiver) = mpsc::channel();
            let (stdout_handle, stderr_handle) =
                get_stdxxx_handles(c.stdout.take(), c.stderr.take(), &sender);

            drop(sender);

            for line in receiver {
                println!("\r\x1b[90m  │\x1b[0m  {}", line);
            }

            let status = c.wait()?;
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();

            if !status.success() {
                log!(warn, "An error occoured. Waiting for changes...");

                break 'outer;
            }

            if i == task_list.len() - 1 {
                log!(info, "Waiting for changes...");

                *child = Some(c);
            }
        }
    }

    Ok(())
}

pub fn start_watcher(task: &Task, config: &Config, clear_terminal_on_restart: bool) -> Result<()> {
    let Some(watch_list) = task.clone().watch.filter(|w| !w.is_empty()) else {
        return Err(anyhow!("No available watch list for the specified task"));
    };

    let (sender, receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(200), None, sender)?;

    // Register the watch_list into the debouncer
    for path in watch_list {
        if let Err(e) = debouncer.watch(Path::new(&path), RecursiveMode::Recursive) {
            return Err(anyhow!(format!("Can't monitor \"{}\": {}", path, e)));
        }
    }

    let mut child: Option<Child> = None;
    let task_list = get_tasks_to_execute(task, config)?;

    if clear_terminal_on_restart {
        clear_terminal()?;
    }

    log!(info, "Starting the selected task...");

    execute(&task_list, config, &mut child)?;

    for event in receiver {
        match event {
            Err(err) => return Err(anyhow!("Error while watching: {:?}", err)),
            Ok(event_list) => {
                for event in event_list {
                    let needs_restart = matches!(
                        event.event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    );

                    if needs_restart {
                        if clear_terminal_on_restart {
                            clear_terminal()?;
                        }

                        log!(info, "File changes detected.");
                        log!(info, "Restarting the task...");

                        execute(&task_list, config, &mut child)?;
                    }
                }
            }
        }
    }

    Ok(())
}
