use crate::tui::state::State;
use anyhow::Result;

pub fn run() -> Result<()> {
    let mut state = State::new();

    let terminal = ratatui::init();
    let result = state.run(terminal);

    ratatui::restore();

    result
}
