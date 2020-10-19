use log::error;
use std::process;

use dotfiles_importer::Importer;

fn main() {
    env_logger::init();

    let importer = Importer::new().unwrap_or_else(|e| {
        error!("Could not create importer: {}", e);
        process::exit(1)
    });

    importer.setup().unwrap_or_else(|e| {
        error!("Setup failed: {}", e);
        process::exit(1)
    })
}
