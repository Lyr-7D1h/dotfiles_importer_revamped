use crate::CONFIG_PATH;
use crate::{importer::config::Config, Importer};
use log::{debug, error, info};
use std::error::Error;
use std::path::Path;

use crate::{BUFFER_SIZE, SOCKET_PATH};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::{fs, io::prelude::*};
use std::{io, thread};

mod handlers;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100))
}

// TODO: Should probably make my own error type
fn make_error(error: Box<dyn Error>, description: &str) -> Box<io::Error> {
    Box::new(io::Error::new(
        io::ErrorKind::Other,
        format!("{}: {}", description, error),
    ))
}

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub fn new() -> io::Result<Server> {
        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(listener) => listener,
            Err(_) => {
                debug!("Could not create listener removing and trying again");
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

    /// Little wrapper around importer listen so you can still send messages without a valid importer
    pub fn listen(&self) -> Result<(), Box<dyn Error>> {
        match Config::from_settings() {
            Ok(config) => {
                let mut importer = match Importer::from_config(config) {
                    Ok(importer) => importer,
                    Err(e) => {
                        return Err(make_error(e, "Could not create importer"));
                    }
                };
                if let Err(e) = importer.setup() {
                    return Err(make_error(e, "Setup failed"));
                }

                if let Err(e) = importer.listen() {
                    return Err(make_error(e, "Could not sync"));
                }
            }
            Err(e) => {
                error!("Could not create config: {}", e);
                loop {
                    if let Ok((stream, _)) = self.listener.accept() {
                        check_messages(stream, |request| get_response_importless(request));
                        // try again
                        return self.listen();
                    }
                    sleep();
                }
            }
        }
        Ok(())
    }

    /// Will listen for messages for 300+ seconds and will then return
    pub fn check_messages_for_300(&self, importer: &mut Importer) -> io::Result<()> {
        let mut iter = 0;
        loop {
            if iter > 3000 {
                return Ok(());
            }

            if let Ok((stream, _)) = self.listener.accept() {
                check_messages(stream, |request| get_response(request, importer));
            }

            iter += 1;
            sleep();
        }
    }
}

fn check_messages<F>(mut stream: UnixStream, op: F)
where
    F: FnOnce(&str) -> Result<String, String>,
{
    let mut buffer = vec![0; BUFFER_SIZE];
    stream
        .read_exact(&mut buffer)
        .expect("Could not read message from cli");

    let request = String::from_utf8(buffer).unwrap();
    let request = request.trim_end_matches("\u{0}");
    info!("Receive from cli: {}", request);

    let response = match op(request) {
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
fn get_response_importless(request_raw: &str) -> Result<String, String> {
    let mut request = request_raw.split(" ");
    match request.next() {
        Some(command) => {
            match command {
                "init" => {
                    if let Some(home_path) = request.next() {
                        if let Err(e) = Config::write("home_path", home_path) {
                            return Err(format!("Could not write home path: {}", e));
                        };
                        if let Some(repository) = request.next() {
                            if let Err(e) = Config::write("repository", repository) {
                                return Err(format!("Could not write repository: {}", e));
                            }
                        }
                        let mut private_key_path = Path::new(home_path).join(".ssh/id_ecdsa");
                        if !private_key_path.exists() {
                            private_key_path = Path::new(home_path).join(".ssh/id_rsa");
                            if !private_key_path.exists() {
                                return Err("Could not find valid ssh key".into());
                            }
                        }
                        if let Err(e) =
                            Config::write("private_key_path", private_key_path.to_str().unwrap())
                        {
                            return Err(format!("Could not write private key path: {}", e));
                        }
                    }
                }
                "config" => match Config::show_raw() {
                    Ok(config) => return Ok(config),
                    Err(e) => return Err(format!("Could not fetch config: {}", e)),
                },
                "set" => {
                    if let Some(arg) = request.next() {
                        if arg.eq("repo") {
                            if let Some(repo) = request.next() {
                                if let Err(e) = Config::write("repository", repo) {
                                    return Err(format!("Could not write home path: {}", e));
                                }
                            }
                        } else if arg.eq("home") {
                            if let Some(path) = request.next() {
                                if let Err(e) = Config::write("home_path", path) {
                                    return Err(format!("Could not write home path: {}", e));
                                }
                            }
                        } else if arg.eq("private_key") {
                            if let Some(path) = request.next() {
                                if let Err(e) = Config::write("private_key_path", path) {
                                    return Err(format!("Could not write home path: {}", e));
                                }
                            }
                        }
                    }
                }
                _ => {
                    let config_err = match Config::from_settings() {
                        Ok(_) => return Ok("Valid config. Setting up..".to_string()),
                        Err(e) => e,
                    };

                    return Err(
                        format!("Dimport is unitialized\nInvalid Config: {}\n\nSee the daemon logs and set the correct values using the commands. \nYou can also manually edit the config at `{}` although this is not recommended.", config_err, CONFIG_PATH)
                    );
                }
            };
            let config_message;
            match Config::from_settings() {
                Ok(_) => config_message = "Valid config. Setting up..".to_string(),
                Err(e) => config_message = format!("Invalid config: {}", e),
            }
            return Ok(format!(
                "Succesfully written to config file.\n\n{}",
                config_message
            ));
        }
        None => return Err("Invalid Command".into()),
    }
}
fn get_response(request: &str, importer: &mut Importer) -> Result<String, String> {
    let mut request = request.split(" ");

    if let Some(command) = request.next() {
        match command {
            "status" => {
                return handlers::status(importer);
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
                    } else if arg.eq("private_key") {
                        if let Some(path) = request.next() {
                            return handlers::set_private_key(path, importer);
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
