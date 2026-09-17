# Grayman

Grayman is CLI and a TUI to execute tasks.

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

## Development

To make the executable available in the current terminal session, you can export it to path with the command:

```
export PATH="$PWD/target/debug:$PATH"
```
