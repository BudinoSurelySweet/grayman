use crate::{log, serializer::essential::init};
use anyhow::Result;

pub fn execute_init() -> Result<()> {
    let result = init(false);

    match result {
        Ok(_) => log!(info, "You succesfully initialized the configuration"),
        Err(err) => {
            log!(error, "{}\n", err);

            let user_wants_to_reinit = inquire::Confirm::new("Do you want to re-initialize?")
                .with_help_message("y/N")
                .with_starting_input("No")
                .prompt()?;

            if user_wants_to_reinit {
                init(true)?;
                log!(info, "\nYou succesfully re-initialized the configuration");
            }
        }
    }

    Ok(())
}
