use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::{env, process};

mod args;
use args::{Args, Ignore};

static SOCKET_PATH: &str = "/tmp/dimport-socket";
static BUFFER_SIZE: usize = 10000;

fn main() {
    let args = Args::from(env::args().collect()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(0)
    });

    let mut stream = UnixStream::connect(SOCKET_PATH).unwrap_or_else(|e| {
        eprintln!("Could not connect to the daemon: {}", e);
        process::exit(1)
    });

    let mut buffer = vec![0; BUFFER_SIZE];
    let mut write = |request: &str| {
        stream.write_all(&raw(request)).unwrap();
    };

    match args {
        Args::Status => {
            write("status");
        }
        Args::Backup => {
            write("backup");
        }
        Args::Config => write("config"),
        Args::Sync => write("sync"),
        Args::Set(set) => match set {
            args::Set::Repository(repo) => write(&format!("set repo {}", repo)),
            args::Set::Home(path) => write(&format!("set home {}", path.to_str().unwrap())),
        },
        Args::Ignore(ignore) => match ignore {
            Ignore::All => write("ignore all"),
            Ignore::Search(regex) => {
                write(&format!("ignore {}", regex));
            }
        },
        Args::Restore(regex) => {
            write(&format!("restore {}", regex));
        }
        Args::Add(path) => {
            write(&format!("add {}", path.to_str().unwrap()));
        }
        Args::Save(description) => {
            if let Some(description) = description {
                write(&format!("save {}", description));
            } else {
                write("save");
            }
        }
    }

    stream.read_exact(&mut buffer).unwrap();
    let response = String::from_utf8(buffer).unwrap();
    if response.starts_with("O") {
        println!("{}", response.strip_prefix("O ").unwrap())
    } else if response.starts_with("E") {
        eprintln!("{}", response.strip_prefix("E ").unwrap());
        process::exit(1)
    }
}

fn raw(request: &str) -> Vec<u8> {
    let mut request = request.to_string().into_bytes();
    request.resize(BUFFER_SIZE, 0);
    request
}
