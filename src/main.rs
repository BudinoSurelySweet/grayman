use grayman::{cli::entry_point::run, log};

fn main() {
    if let Err(err) = run() {
        log!(error, "{}", err);
    }
}
