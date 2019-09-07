use regex::Regex;
use std::collections::VecDeque;
use std::fs::{self, File, ReadDir};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn get_ignores(ignore_file_path: &Path) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    match File::open(ignore_file_path) {
        Ok(f) => {
            let reader = BufReader::new(f);
            for line in reader.lines() {
                let line = line.unwrap();
                result.push(line);
            }
            result
        }
        Err(_) => result,
    }
}

pub fn scan(config: &Path, pattern: String) {
    let mut file = match File::create(config) {
        Ok(f) => f,
        Err(e) => panic!("Could not open config file {}", e),
    };

    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(pattern);

    while let Some(path_str) = queue.pop_front() {
        let current_path: &Path = Path::new(path_str.as_str());
        let dir: ReadDir = match fs::read_dir(current_path) {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to open dir {}", path_str),
        };

        for thing in dir {
            let path: PathBuf = thing.unwrap().path();
            let path_string = String::from(path.to_str().unwrap());
            let is_dotdir: bool = Regex::new(r"/\.").unwrap().is_match(&path_string);

            if path.is_dir() && !is_dotdir {
                let absolute_path = path.canonicalize().unwrap();
                let string = absolute_path.to_str().unwrap().as_bytes();
                file.write(string).unwrap();
                file.write(b"\n").unwrap();

                queue.push_back(String::from(path.as_path().to_str().unwrap()));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_existing_ignore_file() {
        let path = Path::new("test_configs/ignores.txt");
        let ignores = get_ignores(&path);
        let mut expected: Vec<String> = Vec::new();
        expected.push(String::from("node_modules"));
        assert_eq!(ignores, expected);
    }

    #[test]
    fn test_no_ignore_file() {
        let path = Path::new("");
        let ignores = get_ignores(&path);
        let expected: Vec<String> = Vec::new();
        assert_eq!(ignores, expected);
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
    fn test_scan_skips_dot_dirs() {
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

        let v: Vec<&str> = buffer.matches(".git").collect();
        let e: Vec<&str> = Vec::new();
        assert_eq!(v, e);
    }
}
