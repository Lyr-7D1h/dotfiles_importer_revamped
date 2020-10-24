use crate::Importer;
use log::debug;

use crate::{BUFFER_SIZE, SOCKET_PATH};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::{fs, io::prelude::*};
use std::{io, thread};

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100))
}

impl Importer {
    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        let server = server::Server::new(self)?;
        debug!("Created server listener");

        loop {
            info!("Synchronizing..");
            let filenames = self.sync()?;

            // if new changed files notify
            if filenames.len() > self.state.changed_files.len() {
                self.state.changed_files = filenames.clone();
                self.state.save()?;
                let mut body = format!("You have {} changed files.", filenames.len());
                if self.state.suggested_files.len() > 0 {
                    body.push_str(&format!(
                        "\nAnd {} suggested files.",
                        self.state.suggested_files.len()
                    ));
                }
                self.notify(&body)?;
            }

            server.check_messages_for_300()?;
        }
    }
        
    fn listener(&self) -> io::Result<Server> {
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

        Ok(Server {
            listener: listener,
            importer,
        })
    }

    /// Will listen for messages for 300+ seconds and will then return
    pub fn check_messages_for_300(&self) -> io::Result<()> {
        let mut iter = 0;
        loop {
            if iter > 3000 {
                return Ok(());
            }

            if let Ok((stream, _)) = self.listener.accept() {
                debug!("New connection");
                thread::spawn(move || {
                    check_messages(stream);
                });
            }

            iter += 1;
            sleep();
        }
    }
}

fn check_messages(mut stream: UnixStream) {
    let mut buffer = vec![0; BUFFER_SIZE];
    stream
        .read_exact(&mut buffer)
        .expect("Could not read message from cli");

    let request = String::from_utf8(buffer).unwrap().split(" ");

    match request.next() {
        "status" => {}
    }

    stream
        .write_all(&raw("Response"))
        .expect("Could not send response to cli");

    stream.flush().unwrap();
}

fn raw(response: &str) -> Vec<u8> {
    let mut response = response.to_string().into_bytes();
    response.resize(BUFFER_SIZE, 0);
    response
}
