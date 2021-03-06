# Fastjump

Simply navigate between directories.

## Goals

 1. Provide a simple and fast way to switch between directories no matter where you are in relation to that directory
 2. Keep the features minimal, both regarding config and features
 
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

Use as a shell command. It's recommended to use the helper functions below.

### Configure

Edit the configuration file with your favorite editor.
```
 $ vi ~/.fastjump.yml
```

Add the following (or change to fit your needs):
```yaml
# The root directory to start scanning from, this should probably be your home directory
scan_root: /Users/me

# Optional. Save previous visits and prefer often visited folders when switching
previous_visits: /Users/me/.cache/fastjump_visits.yml

# Number of threads to use when scanning directory structure
num_threads: 3

# Names of folders to ignore. Add any large autogenerated folders here
# Folders starting with `.` will always be ignored.
ignores:
  - node_modules
```

### Jump to location

```zsh
 $ cd $(fastjump myproj)
 // Moves to ~/myproject 
```

### Recommended helper function
Add the following function to your shell config (`~/.bashrc`/`~/.bash_profile`/`~/.zshrc`) and call it with `j myproj` to jump.

```zsh
j() { cd $(fastjump $1) }
```

To save previously used directories add this to your `~/.zshrc`:
```zsh
chpwd() {
    fastjump --save-visit $PWD
}
```

## Development

### Publish new release
 
 1. Bump version number in `Cargo.toml`
 2. Run `./updaterelease.sh`
 3. Update release notes on github
