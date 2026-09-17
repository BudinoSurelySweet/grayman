use std::{
    io::{BufRead, BufReader},
    process::{ChildStderr, ChildStdout},
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

pub fn get_stdxxx_handles(
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    sender: &Sender<String>,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let stdout_handle = thread::spawn({
        let sender = sender.clone();

        move || {
            if let Some(stdout) = stdout {
                let reader = BufReader::new(stdout);

                for line in reader.lines().map_while(Result::ok) {
                    let _ = sender.send(line);
                }
            }
        }
    });

    let stderr_handle = thread::spawn({
        let sender = sender.clone();

        move || {
            if let Some(stderr) = stderr {
                let reader = BufReader::new(stderr);

                for line in reader.lines().map_while(Result::ok) {
                    let _ = sender.send(line);
                }
            }
        }
    });

    (stdout_handle, stderr_handle)
}
