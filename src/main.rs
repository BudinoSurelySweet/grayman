use grayman::{cli::run::run, error};

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
    }
}
