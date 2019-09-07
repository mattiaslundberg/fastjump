use fuzzy_matcher::skim::fuzzy_match;
use std::collections::VecDeque;
use std::fs::{self, File, ReadDir};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
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
    let mut file = match File::create(config) {
        Ok(f) => f,
        Err(e) => panic!("Could not open file {}", e),
    };

    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(pattern);

    while let Some(path_str) = queue.pop_front() {
        let path: &Path = Path::new(path_str.as_str());
        let dir: ReadDir = match fs::read_dir(path) {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to open dir {}", path_str),
        };

        for thing in dir {
            let path: PathBuf = thing.unwrap().path();
            if path.is_dir() {
                let absolute_path = path.canonicalize().unwrap();
                let string = absolute_path.to_str().unwrap().as_bytes();
                file.write(string).unwrap();
                file.write(b"\n").unwrap();

                queue.push_back(String::from(path.as_path().to_str().unwrap()));
            }
        }
    }
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

    if best_score < 10 {
        best_result = String::from(".");
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
    #[should_panic(expected = "Could not open ")]
    fn test_scan_non_existing_dir() {
        let path = Path::new("/tmp/asdf/");
        let pattern = String::new();
        scan(&path, pattern);
    }

    #[test]
    fn test_scan_non_recursive_dir() {
        let path = Path::new("/tmp/scannonrecursive");
        let pattern = String::from("test_configs/");
        scan(&path, pattern);

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => panic!("Could not open file {}", e),
        };
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();

        reader.read_to_string(&mut buffer).unwrap();

        let v: Vec<&str> = buffer.matches("empty").collect();
        assert_eq!(v, ["empty"]);
    }

    #[test]
    fn test_scan_recursive_dir() {
        let path = Path::new("/tmp/scan_recursive");
        let pattern = String::from(".");
        scan(&path, pattern);

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => panic!("Could not open file {}", e),
        };
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();

        reader.read_to_string(&mut buffer).unwrap();

        let v: Vec<&str> = buffer.matches("empty").collect();
        assert_eq!(v, ["empty"]);
    }

    #[test]
    #[should_panic(expected = "Could not open")]
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
        assert_eq!(change(&path, pattern), ".")
    }
}
