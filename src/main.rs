mod worker;
mod worker_io;
mod worker_task;
mod worker_transport;

fn main() {
    if let Err(error) = worker::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
