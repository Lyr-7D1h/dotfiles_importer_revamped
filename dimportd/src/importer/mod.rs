use crate::Server;
use log::info;
use notify_rust;
use notify_rust::{Notification, NotificationHandle};
use std::{error::Error, io};

pub struct Importer {
    pub state: State,
    pub config: Config,
}

pub mod config;
mod link;

mod sync;

use config::Config;

pub mod state;
use state::State;

impl Importer {
    pub fn new() -> Result<Importer, Box<dyn Error>> {
        let state = State::get()?;
        let config = Config::from_settings()?;
        Ok(Importer { state, config })
    }
    pub fn from_config(config: Config) -> Result<Importer, Box<dyn Error>> {
        let state = State::get()?;
        Ok(Importer { state, config })
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        let server = Server::new()?;

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
