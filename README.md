# Todo

Todo is a cli tool to report TODO comments to your issue tracker (for example Github).
This project was created to learn about the Rust language and the idea was stolen from
[snitch](https://github.com/tsoding/snitch).

It supports reporting todos to Github and [gitea](https://gitea.io/).

## TODO Format

### Unreported

```
// TODO: This is an exaple TODO
```

The todo can have any prefix and starts with a *keyword* defined in the config (Default: TODO).
The body will be used as the title in the issue.

### Reported

```
// TODO(#123): This is a reported TODO
```

A reported todo has the issue number of the reported issue.

### Comments

```
// TODO: This is a TODO
// That has also comments
```

The comments must start with the same prefix as the body and will be added to the body of the issue.

## Usage

To try it out you have to install the [Rust toolchain](https://www.rust-lang.org/tools/install)
and compile it with cargo.

```sh
USAGE:
    todo [SUBCOMMAND]

FLAGS:
    -h, --help    Prints help information

SUBCOMMANDS:
    files     Prints all files, filtered after the config
    help      Prints this message or the help of the given subcommand(s)
    list      Lists all (un)reported
    purge     Purges all closed TODOs
    report    Reports all new TODOs
```

## Config

You can have a global and a local config file for you project.
The global is saved as `todo.yml` in your config folder
(on linux it's `~/.config/todo.yml`, for other systems check out [dirs](https://github.com/dirs-dev/dirs-rs)).
The local one has to be named `.todo.yml` at the root of your project.

If both configs have the same fields the fields from the local config are always used.
But if the ignore mode is the same, the patterns are concatenated.

```yaml
backend: Gitea | Github
user: Username of the owner
repo: Name of the repo
token: Token to authenticate
url: Location of the gitea instance (Needed for Gitea)

ignore_mode: Blacklist | Whitelist (Default Blacklist)
patterns: List of Patterns to black or whitelist (Optional)
keywords: List of Keywords to search in files (Default [TODO])
```

### Example

```yaml
backend: Github
user: r0m4n27
repo: todo-rs
token: <my token>

patterns:
    - .git/
    - target/
keywords:
    - TODO
    - BUG
```
