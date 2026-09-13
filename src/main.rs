use grayman::{cli::entry_point::run, error};

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
    }
}
