# Roadmap

...

## Now

- Refining the UX of the TUI.

## Next

- Write the README.md
- Add some documentation.

## Later

- Aggiungere un menu di conferma per quando l'utente fa alcune azioni (tipo quit).
- Add a task's metadata viewer
- Add a way to add/edit/remove tasks in the TUI's tasks panel.
- Add a way to add/edit/remove variables in the TUI's variables panel.
- Add a settings panel to the TUI.
- Add a help panel (for suggestion on how to use the TUI) to the TUI.
- Add a keybinds panel to the TUI.
- Try integrating a terminal inside the TUI. (maybe with the crate `tui-term`)
- Add a way to switch layout in the TUI. (e.g. 3 panel layout, 1 panel layout)
- Parallelize independent tasks.
- Smart caching by tracking files (skip tasks that don't need to be runned).
- Add a config searcher: First grayman should search in the current directory for .grayman after that it should search in other directory.
- Dynamic environment variables (resolved at runtime).
- Export grayman's tasks to json for editors to use it.
- Support for `.env` file (crate `dotenvy`)
- Add a `grayman.local.toml` for local tasks (not tracked by git)
