use log::info;
use notify_rust;
use notify_rust::{Notification, NotificationHandle};
use std::{error::Error, io};

mod link;
mod server;
mod sync;
mod util;

mod config;
use config::Config;

mod state;
use state::State;

static SOCKET_PATH: &str = "/tmp/dimport-socket";
static BUFFER_SIZE: usize = 10000;

static CONFIG_PATH: &str = "../config.json";
static STATE_PATH: &str = "../state.json";
static REPOSITORY_DIR: &str = "../repository";
static BACKUP_DIR: &str = "../backup";
static PRIVATE_KEY_PATH: &str = "/home/lyr/.ssh/id_ecdsa";

pub struct Importer {
    state: State,
    config: Config,
}

impl Importer {
    pub fn new() -> Result<Importer, Box<dyn Error>> {
        let state = State::get()?;
        let config = Config::from_settings()?;
        Ok(Importer { state, config })
    }
    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        let server = server::Server::new()?;

        loop {
            let changes = self.sync()?;

            // if new changed files notify
            if changes.len() > self.state.changed_files.len() {
                self.state.changed_files = changes.clone();
                self.state.save()?;
                let mut body = format!("You have {} changed files.", changes.len());
                if self.state.suggested_files.len() > 0 {
                    body.push_str(&format!(
                        "\nAnd {} suggested files.",
                        self.state.suggested_files.len()
                    ));
                }
                self.notify(&body)?;
            }

            server.check_messages_for_300(self)?;
        }
    }

    pub fn notify(&self, body: &str) -> notify_rust::error::Result<NotificationHandle> {
        info!("Notify: {}", body);
        Notification::new()
            .summary("Dotfiles Importer")
            .body(body)
            .show()
    }

    pub fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.state.initialized {
            info!("Setting up...");
            self.backup().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Could not backup files: {}", e),
                )
            })?;

            self.link().map_err(|e| {
                info!("Could not link files: {}", e);
                let err = self
                    .restore()
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Could not restore from backup: {}", e),
                        )
                    })
                    .err();

                if let Some(err) = err {
                    return err;
                } else {
                    return io::Error::new(io::ErrorKind::Other, format!("Linking failed: {}", e));
                }
            })?;

            self.intitialize_mapped()?;

            self.state.suggested_files = vec![];
            self.state.save()?;

            self.state.initialized = true;
            self.state.save()?;
            return Ok(());
        }
        info!("Already setup");

        Ok(())
    }
}
