use log::error;
use std::process;

use dimportd::Server;

fn main() {
    env_logger::init();

    let server = Server::new().unwrap_or_else(|e| {
        error!("Could not create server: {}", e);
        process::exit(1)
    });
    if let Err(e) = server.listen() {
        error!("{}", e);
        process::exit(1)
    }
}
