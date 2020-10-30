use std::env;

fn main() {
    let socket_path = env::var("SOCKET_PATH").unwrap_or("/tmp/dimportd.socket".into());
    println!("cargo:rustc-env=SOCKET_PATH={}", socket_path);
    let config_path = env::var("CONFIG_PATH").unwrap_or("config.json".into());
    println!("cargo:rustc-env=CONFIG_PATH={}", config_path);
    let state_path = env::var("STATE_PATH").unwrap_or("state.json".into());
    println!("cargo:rustc-env=STATE_PATH={}", state_path);
    let repostiory_dir = env::var("REPOSITORY_DIR").unwrap_or("repository".into());
    println!("cargo:rustc-env=REPOSITORY_DIR={}", repostiory_dir);
    let backup_dir = env::var("BACKUP_DIR").unwrap_or("backup".into());
    println!("cargo:rustc-env=BACKUP_DIR={}", backup_dir);
}
