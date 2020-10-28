# Dotfiles Importer

Give a repository and sync all dotfiles.

It will watch for changes and ask if you want to add files to your dotfiles when directly added to one of the folders.

Otherwise you can add folders using the cli.

## Installation

### Arch

```
makepkg -si
```

## Usage

Use dimport cli to configure the service

dimport is the cli used for interacting with the service

dimportd is the service this will receive commands from cli and synchronize everything

```
Dotfiles Import
Lyr-7D1h <lyr-7d1h@pm.me>
Usage:
    dimport <command> [<args>]

Commands:
    status                          Show changed files and show suggested files.
    backup                          Backup current conflicting dotfiles, will override if there already is an backup
    config                          Return current configuration
    sync                            Synchronize files right now (otherwise being run every ~5 min)
    set [repo|home] [<url>|<path>]  Configure the dotfiles importer
    ignore [all|<regex>]            If you want to ignore all suggested files or only by regex
    restore <regex>                 Restore a removed or changed file
    add <path>                      Add a file or directory to the repository
    save [<message>]                Save current settings and give an optional description of changed files
```

## Notes

No save/push support for `https://` repositories.
Mind that I had issues with rsa ssh keys ecdsa works fine.
