use regex::Regex;
use std::collections::{HashSet, VecDeque};
use std::fs::{self, File, ReadDir};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn get_ignores(ignore_file_path: &Path) -> HashSet<String> {
    let mut result: HashSet<String> = HashSet::new();
    match File::open(ignore_file_path) {
        Ok(f) => {
            println!("Loaded ignore_file {:?}", ignore_file_path);
            let reader = BufReader::new(f);
            for line in reader.lines() {
                let line = line.unwrap();
                result.insert(line);
            }
            result
        }
        Err(_) => {
            println!("Could not load ignore_file {:?}", ignore_file_path);
            result
        }
    }
}

pub fn scan(config: &Path, pattern: String) {
    let mut file = match File::create(config) {
        Ok(f) => f,
        Err(e) => panic!("Could not open config file {}", e),
    };

    let mut ignore_path: PathBuf = PathBuf::from(pattern.as_str());
    ignore_path.push(".fjignore");

    let ignores: HashSet<String> = get_ignores(ignore_path.as_path());

    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(pattern);

    let re = Regex::new(r"/\.").unwrap();

    while let Some(path_str) = queue.pop_front() {
        let current_path: &Path = Path::new(path_str.as_str());
        let dir: ReadDir = match fs::read_dir(current_path) {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to open dir {}", path_str),
        };

        for thing in dir {
            let path: PathBuf = thing.unwrap().path();
            if !path.is_dir() {
                continue;
            };

            let path_str = path.to_str().unwrap();
            let path_string: String = String::from(path_str);

            if re.is_match(&path_string) {
                continue;
            };

            let mut parts: Vec<&str> = path_str.split('/').collect();
            let folder: &str = parts.pop().unwrap();

            if ignores.contains(folder) {
                continue;
            }

            file.write(path_str.as_bytes()).unwrap();
            file.write(b"\n").unwrap();

            queue.push_back(String::from(path_str));
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
        let mut expected: HashSet<String> = HashSet::new();
        expected.insert(String::from("node_modules"));
        assert_eq!(ignores, expected);
    }

    #[test]
    fn test_no_ignore_file() {
        let path = Path::new("");
        let ignores = get_ignores(&path);
        let expected: HashSet<String> = HashSet::new();
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
