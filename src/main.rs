use std::process;

use dotfiles_importer::{backup, link, restore, Config};

fn main() {
    let config = Config::from_settings().unwrap_or_else(|e| {
        eprintln!("Error from reading settings: {}", e);
        process::exit(1)
    });

    setup(&config);
}

fn setup(config: &Config) {
    println!("Config loaded");
    backup(&config).unwrap_or_else(|e| {
        eprintln!("Could not backup files: {}", e);
        process::exit(1)
    });

    link(&config).unwrap_or_else(|e| {
        eprintln!("Could not link repository files: {}", e);
        eprintln!("Restoring from backup");
        restore(&config).unwrap_or_else(|e| {
            eprintln!("Could not restore from backup: {}", e);
        });
        process::exit(1)
    });
}
