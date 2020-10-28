use crate::Importer;
use log::{debug, info};

use crate::{BUFFER_SIZE, SOCKET_PATH};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::{fs, io::prelude::*};
use std::{io, thread};

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

        debug!("Created server listener");

        Ok(Server { listener: listener })
    }

    /// Will listen for messages for 300+ seconds and will then return
    pub fn check_messages_for_300(&self, importer: &mut Importer) -> io::Result<()> {
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

fn check_messages(mut stream: UnixStream, importer: &mut Importer) {
    let mut buffer = vec![0; BUFFER_SIZE];
    stream
        .read_exact(&mut buffer)
        .expect("Could not read message from cli");

    let request = String::from_utf8(buffer).unwrap();
    let request = request.trim_end_matches("\u{0}");
    info!("Receive from cli: {}", request);

    let response = match get_response(request, importer) {
        Ok(pos_res) => {
            info!("Ok Response: {}", pos_res);
            format!("O {}", pos_res)
        }
        Err(neg_res) => {
            info!("Error Response: {}", neg_res);
            format!("E {}", neg_res)
        }
    };

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

fn get_response(request: &str, importer: &mut Importer) -> Result<String, String> {
    let mut request = request.split(" ");

    if let Some(command) = request.next() {
        match command {
            "status" => {
                return handlers::status(importer);
            }
            "backup" => {
                return handlers::backup(importer);
            }
            "config" => {
                return handlers::config(importer);
            }
            "sync" => {
                return handlers::sync(importer);
            }
            "set" => {
                if let Some(arg) = request.next() {
                    if arg.eq("repo") {
                        if let Some(repo) = request.next() {
                            return handlers::set_repository(repo, importer);
                        }
                    } else if arg.eq("home") {
                        if let Some(home) = request.next() {
                            return handlers::set_home(home, importer);
                        }
                    }
                }
            }
            "ignore" => {
                if let Some(arg) = request.next() {
                    if arg.eq("all") {
                        return handlers::ignore_all(importer);
                    } else {
                        return handlers::ignore_regex(arg, importer);
                    }
                }
            }
            "restore" => {
                if let Some(arg) = request.next() {
                    return handlers::restore(arg, importer);
                }
            }
            "add" => {
                if let Some(arg) = request.next() {
                    return handlers::add(arg, importer);
                }
            }
            "save" => {
                if let Some(arg) = request.next() {
                    let description =
                        format!("{} {}", arg, request.collect::<Vec<&str>>().join(" "));
                    return handlers::save(Some(&description), importer);
                } else {
                    return handlers::save(None, importer);
                }
            }
            _ => return Err("Invalid command".into()),
        }
    } else {
        return Err("Empty command".into());
    }

    return Err("Could not find command".into());
}
