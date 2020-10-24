use crate::Importer;
use log::debug;

use crate::{BUFFER_SIZE, SOCKET_PATH};
use std::os::unix::net::UnixStream;
use std::{fs, io::prelude::*};
use std::{io, thread};
use std::{os::unix::net::UnixListener, sync::Mutex};

mod handlers;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100))
}

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub fn new() -> io::Result<Server> {
        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(listener) => listener,
            Err(_) => {
                fs::remove_file(SOCKET_PATH)?;
                UnixListener::bind(SOCKET_PATH)?
            }
        };

        listener
            .set_nonblocking(true)
            .expect("Could not set listener to non_blocking");

        Ok(Server { listener: listener })
    }

    /// Will listen for messages for 300+ seconds and will then return
    pub fn check_messages_for_300(&self, importer: &Importer) -> io::Result<()> {
        let mut iter = 0;
        loop {
            if iter > 3000 {
                return Ok(());
            }

            if let Ok((stream, _)) = self.listener.accept() {
                debug!("New connection");
                // let importer = Mutex::new(importer);
                check_messages(stream, importer);
                // });
            }

            iter += 1;
            sleep();
        }
    }
}

fn check_messages(mut stream: UnixStream, importer: &Importer) {
    let mut buffer = vec![0; BUFFER_SIZE];
    stream
        .read_exact(&mut buffer)
        .expect("Could not read message from cli");

    let request = String::from_utf8(buffer).unwrap();
    let request = request.trim_end_matches("\u{0}");
    debug!("Receive from cli: {}", request);
    let mut request = request.split(" ");

    let response;
    if let Some(command) = request.next() {
        println!("{:?}", command);
        match command {
            "status" => {
                response = handlers::status(importer);
            }
            _ => response = "Invalid command".into(),
        }
    } else {
        response = "Empty command".into();
    }

    stream
        .write_all(&raw(&response))
        .expect("Could not send response to cli");

    stream.flush().unwrap();
}

fn raw(response: &str) -> Vec<u8> {
    let mut response = response.to_string().into_bytes();
    response.resize(BUFFER_SIZE, 0);
    response
}
