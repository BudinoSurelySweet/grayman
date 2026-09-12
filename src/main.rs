use grayman::{cli::run, error};

fn main() {
    if let Err(err) = run::run() {
        error!("{}", err);
    }
}
