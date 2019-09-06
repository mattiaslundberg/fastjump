use fuzzy_matcher::skim::fuzzy_match;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Cli {
    #[structopt(short, long)]
    scan: bool,

    pattern: String,
}

fn main() {
    let args = Cli::from_args();
    let config_file = get_config_file();
    let config = Path::new(config_file.as_str());

    if args.scan {
        scan(config, args.pattern);
    } else {
        change(config, args.pattern);
    }
}

fn get_config_file() -> String {
    match std::env::var("FASTJUMP_CONFIG") {
        Ok(val) => val,
        Err(_e) => String::from("~/.fastjump"),
    }
}

fn scan(config: &Path, pattern: String) {
    println!("Scanning {}", pattern)
}

fn change(config: &Path, pattern: String) -> String {
    let file = match File::open(config) {
        Ok(f) => f,
        Err(e) => panic!("Could not open file {}", e),
    };

    let reader = BufReader::new(file);
    let mut best_score = 0;
    let mut best_result: String = String::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let score = match fuzzy_match(&line, &pattern) {
            Some(s) => s,
            None => 0,
        };

        if score > best_score {
            best_score = score;
            best_result = line;
        }
    }

    println!("{}", best_result);
    best_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_file() {
        assert_eq!(get_config_file(), "~/.fastjump");
        std::env::set_var("FASTJUMP_CONFIG", "/tmp/fastjump_test");
        assert_eq!(get_config_file(), "/tmp/fastjump_test");
    }

    #[test]
    #[should_panic(expected = "Could not open file")]
    fn test_change_non_existing_config() {
        let path = Path::new("/tmp/nonexistingfile");
        let pattern = String::new();
        change(&path, pattern);
    }

    #[test]
    fn test_good_match() {
        let path = Path::new("test_configs/good");
        let pattern = String::from("good");
        assert_eq!(change(&path, pattern), "/tmp/good/hello")
    }

    #[test]
    fn test_no_match() {
        let path = Path::new("test_configs/good");
        let pattern = String::from("nonexisting");
        assert_eq!(change(&path, pattern), "")
    }
}
