# Fastjump

## Goals

 1. Fast way to switch directories using shell aliases
 2. Keep it simple, minimal featureset
 
## Installation

### MacOS (homebrew)

```zsh
 $ brew install mattiaslundberg/fastjump/fastjump
```

### Manual

Requires [rust](https://www.rust-lang.org) to be installed.

```zsh
 $ git clone git@github.com:mattiaslundberg/fastjump.git
 $ cd fastjump
 $ cargo install --root /usr/local/bin/ --path .
```
 
## Usage

Use as a shell command.

### Create an ignore file

```
 $ vi ~/.fjignore # or whereever you want to keep your scan root
```

Folders starting with `.` will always be ignored, it's a good idea to add `node_modules` and other large folders here.

### Build the database with possible targets

```zsh
 $ fastjump --scan ~
```

This will create a database file at `~/.fastjump`, to change the location set the environment variable `FASTJUMP_CONFIG`.

### Jump to location

```zsh
 $ cd $(fastjump myproj)
 // Moves to ~/myproject (assuming it's in the db)
```

### Recommended helper function
Add the following function to your shell config (`~/.bashrc`/`~/.bash_profile`/`~/.zshrc`) and call it with `j myproj` to jump.

```zsh
 $ j() { cd $(fastjump $1) }
```
