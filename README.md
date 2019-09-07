# Fastjump

## Goals

 1. Fast way to switch directories
 2. Keep it simple, minimal featureset
 
## Installation

### MacOS
```zsh
 $ brew install mattiaslundberg/fastjump/fastjump
```
 
## Usage

Use in shell

Build database:
```zsh
 $ fastjump --scan ~
```

Jump to location:
```zsh
 $ cd $(fastjump myproj)
 // Moves to ~/myproject (assuming it's in the db)
```

Recommended helper function (add this to your shell config):
```zsh
 $ j() { cd $(fastjump $1) }
```
