use regex::Regex;
use std::path::PathBuf;
use std::{env, error::Error};

pub enum Ignore {
    All,
    Search(Regex),
}
pub enum Set {
    Repository(String),
    Home(PathBuf),
    PrivateKey(PathBuf),
}

pub enum Args {
    Init((PathBuf, Option<String>)),
    Status,
    Backup,
    Config,
    Sync,
    Set(Set),
    Ignore(Ignore),
    Restore(Regex),
    Add(PathBuf),
    Save(Option<String>),
}

impl Args {
    pub fn from(args: Vec<String>) -> Result<Args, Box<dyn Error>> {
        let mut args = args.iter();

        // Skip first argument
        args.next();

        if let Some(command) = args.next() {
            match command.as_str() {
                "init" => {
                    let home_path = match env::var("HOME") {
                        Ok(home) => PathBuf::from(home),
                        Err(_) => return Err("Could not fetch HOME env variable".into()),
                    };
                    if !home_path.is_absolute() {
                        return Err("Home path is not absolute".into());
                    }
                    let repository = match args.next() {
                        Some(repo) => Some(repo.to_owned()),
                        None => None,
                    };
                    return Ok(Args::Init((home_path, repository)));
                }
                "status" => return Ok(Args::Status),
                "backup" => return Ok(Args::Backup),
                "config" => return Ok(Args::Config),
                "sync" => return Ok(Args::Sync),
                "set" => {
                    if let Some(arg) = args.next() {
                        if arg.eq("repo") {
                            if let Some(repo) = args.next() {
                                return Ok(Args::Set(Set::Repository(repo.clone())));
                            }
                        } else if arg.eq("home") {
                            if let Some(home) = args.next() {
                                let path = PathBuf::from(home);
                                if !path.is_absolute() {
                                    return Err("Please give the absolute path".into());
                                }
                                if !path.exists() {
                                    return Err("Path does not exist".into());
                                }
                                return Ok(Args::Set(Set::Home(path)));
                            }
                        } else if arg.eq("private_key") {
                            if let Some(path) = args.next() {
                                let path = PathBuf::from(path);
                                if !path.is_absolute() {
                                    return Err("Please give the absolute path".into());
                                }
                                if !path.exists() {
                                    return Err("Path does not exist".into());
                                }
                                return Ok(Args::Set(Set::PrivateKey(path)));
                            }
                        }
                    }
                }
                "ignore" => {
                    if let Some(arg) = args.next() {
                        if arg.eq(&"all") {
                            return Ok(Args::Ignore(Ignore::All));
                        } else {
                            let regex = Regex::new(arg)?;
                            return Ok(Args::Ignore(Ignore::Search(regex)));
                        }
                    }
                }
                "restore" => {
                    if let Some(arg) = args.next() {
                        let regex = Regex::new(arg)?;
                        return Ok(Args::Restore(regex));
                    }
                }
                "add" => {
                    if let Some(arg) = args.next() {
                        let path = PathBuf::from(arg);
                        if !path.exists() {
                            return Err("Path does not exist".into());
                        }
                        return Ok(Args::Add(path));
                    }
                }
                "save" => {
                    let description = args
                        .map(|a| a.to_owned())
                        .collect::<Vec<String>>()
                        .join(" ");
                    if description.len() > 0 {
                        return Ok(Args::Save(Some(description)));
                    } else {
                        return Ok(Args::Save(None));
                    }
                }
                _ => {}
            }
        }
        return Err(r#"Dotfiles Import
Lyr-7D1h <lyr-7d1h@pm.me>
Usage:
    dimport <command> [<args>]

Unitialized state commands:
    init [<url>]                                Load config.json with sane defaults and optionally give the repository aswell (will only work when no config setup)
    set [repo|home|private_key] [<url>|<path>]  Configure the dotfiles importer
    config                                      Return current configuration

Commands:
    status                                      Show changed files and show suggested files.
    backup                                      Backup current conflicting dotfiles, will override if there already is an backup
    config                                      Return current configuration
    sync                                        Synchronize files right now (otherwise being run every ~5 min)
    set [repo|home|private_key] [<url>|<path>]  Configure the dotfiles importer
    ignore [all|<regex>]                        If you want to ignore all suggested files or only by regex
    restore <regex>                             Restore a removed or changed file
    add <path>                                  Add a file or directory to the repository
    save [<message>]                            Save current settings and give an optional description of changed files
"#
            .into());
    }
}
