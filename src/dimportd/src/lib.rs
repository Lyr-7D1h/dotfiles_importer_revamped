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

pub const BUFFER_SIZE: usize = 10000;

pub const SOCKET_PATH: &str = "/run/dimportd.socket";

pub const CONFIG_PATH: &str = "/etc/dimport/config.json";
pub const STATE_PATH: &str = "/var/lib/dimport/state.json";
pub const REPOSITORY_DIR: &str = "/var/lib/dimport/repository";
pub const BACKUP_DIR: &str = "/var/lib/dimport/backup";

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
            self.sync_and_notify()?;

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

            self.link_all().map_err(|e| {
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
            self.state.differences = vec![];
            self.state.save()?;

            self.state.initialized = true;
            self.state.save()?;
            return Ok(());
        }
        info!("Already setup");

        Ok(())
    }
}
