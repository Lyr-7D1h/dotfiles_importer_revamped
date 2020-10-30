use std::env;

fn main() {
    let socket_path = env::var("SOCKET_PATH").unwrap_or("/tmp/dimportd.socket".into());
    println!("cargo:rustc-env=SOCKET_PATH={}", socket_path);
}
