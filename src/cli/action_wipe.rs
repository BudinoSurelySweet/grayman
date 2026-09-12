use crate::serializer::essential::wipe;
use anyhow::Result;

pub fn execute_wipe() -> Result<()> {
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
