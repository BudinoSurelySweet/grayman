# Grayman

Grayman is CLI and a TUI to execute tasks.

## Installation

...

## How to use

- If executed without any arguments or flags, Grayman will trigger its default behavior (retrieving and running the last executed task from the cache).

    ```
    grayman [TASK_NAME] [OPTIONS]
    ```

- Pass the task name as a positional argument to start it immediately or launch an interactive terminal menu to browse, select, and run available tasks. (Note: This flag conflicts with and cannot be used if a task name is provided)

    ```
    grayman build
    ```

    ```
    grayman --select
    ```

- Sets up the current directory for Grayman. It creates the configuration file (grayman.toml), prepares the hidden cache directory, and updates .gitignore to prevent tracking local state.

    ```
    grayman --init
    ```

- Launches Grayman's TUI (Terminal User Interface). (Note: This feature is heavely work in progess).

    ```
    grayman --tui
    ```

## Anatomy of the configuration file

Grayman uses a configuration file in TOML format, named `grayman.toml` and located in the root of your project. The file is structured around two main concepts: global environment variables and sequential task definitions.

Below is an example configuration for building and running a C project:

```toml
[env]
cc = "gcc"
output_name = "output"
output_folder = "build"

[[tasks]]
name = "build"
commands = ["$cc *.c -o ../$output_folder/$output_name"]
cwd = "src"
watch = ["src"]

[[tasks]]
name = "run"
commands = ["./$output_name"]
depends_on = ["build"]
cwd = "build"
watch = ["src"]
```

### `[env]` (Global Variables)

An optional block used to define environment variables that can be reused throughout the entire file.
These variables can be referenced inside commands or paths using the `${variable_name}` or `$variable_name` syntax. Grayman handles their safe, cross-platform expansion before executing the commands.

### `[[tasks]]` (Task Definition)

Each `[[tasks]]` block represents an independent action that Grayman can execute (either via the standard CLI or by selecting it from the TUI).

| Field         | Type            | Description                                                                                                                                                                                                    |
| :------------ | :-------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`        | String          | **(Required)** The unique name of the task. This is the name used to invoke it (e.g., `grayman build`).                                                                                                        |
| `description` | String          | _(Optional)_ A brief description of the task. Useful for documenting what the command does.                                                                                                                    |
| `commands`    | Array of String | **(Required)** A list of commands to execute. Grayman natively supports the expansion of variables defined in `[env]` or in the OS environment, as well as cross-platform globbing (e.g., `*.c` or `**/*.rs`). |
| `env`         | Table           | _(Optional)_ Task-specific environment variables. These local variables take precedence over the global `[env]` variables with the same name.                                                                  |
| `cwd`         | String          | _(Optional)_ The Current Working Directory. If specified, Grayman will change into this directory before executing the task's commands (e.g., `cwd = "src"` saves you from writing `cd src && ...`).           |
| `shell`       | Boolean         | _(Optional)_ Whether to execute the commands inside a shell. Defaults to `true`. If set to `false`, Grayman will run the executable directly, bypassing shell expansion and overhead.                          |
| `depends_on`  | Array of String | _(Optional)_ Defines a dependency chain. Grayman ensures these prerequisite tasks are executed **sequentially** before starting the current task (e.g., running `build` before `run`).                         |
| `watch`       | Array of String | _(Optional)_ A list of paths (files or directories) to monitor. When Grayman is launched with the `--watch` flag, any changes or saves to these files will instantly interrupt and restart the task.           |

## Known Limitations

1. Due to the way Grayman spawns processes, some commands may lose their colored output. If you encounter this issue, check if the command you are using has a specific flag to force colors. Below there is a table with some useful flags.

    | Command     | Flag                         |
    | ----------- | ---------------------------- |
    | `cargo`     | `--color=always`             |
    | `gcc`/`g++` | `-fdiagnostics-color=always` |
    | `clang`     | `-fcolor-diagnostics`        |
    | `git`       | `-c color.ui=always`         |
    | `npm`       | `--color=always`             |
    | `pytest`    | `--color=yes`                |

2. The TUI is not fully capable of doing everything as the CLI.
3. The `cwd` and `watch` fields can't contain path to expands or glob pattern at the moment.

## Development

To make the executable available in the current terminal session, you can export it to path with the command:

```
export PATH="$PWD/target/debug:$PATH"
```
