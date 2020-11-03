mod server;
pub use server::Server;

mod util;

mod importer;
pub use importer::Importer;

pub const BUFFER_SIZE: usize = 10000;

pub const SOCKET_PATH: &str = env!("SOCKET_PATH");
pub const CONFIG_PATH: &str = env!("CONFIG_PATH");
pub const STATE_PATH: &str = env!("STATE_PATH");
pub const REPOSITORY_DIR: &str = env!("REPOSITORY_DIR");
pub const BACKUP_DIR: &str = env!("BACKUP_DIR");
