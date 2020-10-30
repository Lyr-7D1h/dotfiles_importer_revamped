mod server;
pub use server::Server;

mod util;

mod importer;
pub use importer::Importer;

pub const BUFFER_SIZE: usize = 10000;

pub const SOCKET_PATH: &str = "/tmp/dimportd.socket";

pub const CONFIG_PATH: &str = "config.json";
pub const STATE_PATH: &str = "state.json";
pub const REPOSITORY_DIR: &str = "repository";
pub const BACKUP_DIR: &str = "backup";
