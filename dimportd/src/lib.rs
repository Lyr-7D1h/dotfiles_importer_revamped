mod server;
pub use server::Server;

mod util;

mod importer;
pub use importer::Importer;

pub const BUFFER_SIZE: usize = 10000;

// pub const SOCKET_PATH: &str = "/run/dimportd.socket";
pub const SOCKET_PATH: &str = "/tmp/dimportd.socket";

pub const CONFIG_PATH: &str = "/etc/dimport/config.json";
pub const STATE_PATH: &str = "/var/lib/dimport/state.json";
pub const REPOSITORY_DIR: &str = "/var/lib/dimport/repository";
pub const BACKUP_DIR: &str = "/var/lib/dimport/backup";
