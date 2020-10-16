use std::process;

use dotfiles_importer::{link, Config};

fn main() {
    let config = Config::from_settings().unwrap_or_else(|e| {
        eprintln!("Error from reading settings: {}", e);
        process::exit(1)
    });

    println!("Config loaded");
    //backup

    link(&config).unwrap_or_else(|e| {
        // remove links and restore from backup
        eprintln!("Could not link repository files: {}", e);
        process::exit(1)
    })
}
