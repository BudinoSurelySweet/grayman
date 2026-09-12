use fucina::{cli::cli, error};

fn main() {
    if let Err(err) = cli::run() {
        error!("{}", err);
    }
}
