use regex::Regex;
use std::error::Error;
use std::path::PathBuf;

pub enum Ignore {
    All,
    Search(Regex),
}
pub enum Set {
    Repository(String),
    Home(PathBuf),
}

pub enum Args {
    Status,
    Backup,
    Config,
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
                "status" => return Ok(Args::Status),
                "backup" => return Ok(Args::Backup),
                "config" => return Ok(Args::Config),
                "set" => {
                    if let Some(arg) = args.next() {
                        if arg.eq("repo") {
                            if let Some(repo) = args.next() {
                                return Ok(Args::Set(Set::Repository(repo.clone())));
                            }
                        } else if arg.eq("home") {
                            if let Some(home) = args.next() {
                                let path = PathBuf::from(home);
                                if !path.exists() {
                                    return Err("Path does not exist".into());
                                }
                                return Ok(Args::Set(Set::Home(path)));
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
                    let mut description = String::new();
                    while let Some(arg) = args.next() {
                        description.push_str(arg);
                    }
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

Commands:
    status                          Show changed files and show suggested files.
    backup                          Backup current conflicting dotfiles, will override if there already is an backup
    config                          Return current configuration
    set [repo|home] [<url>|<path>]  Configure the dotfiles importer
    ignore [all|<regex>]            If you want to ignore all suggested files or only by regex
    restore <regex>                 Restore a removed or changed file
    add <path>                      Add a file or directory to the repository
    save [<message>]                Save current settings and give an optional description of changed files
"#
            .into());
    }
}