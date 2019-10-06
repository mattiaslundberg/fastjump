use crate::config::Config;
use fuzzy_matcher::skim::fuzzy_match;
use std::collections::VecDeque;
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn matcher(config: Config, pattern: String) -> String {
    let pattern = Arc::new(pattern.chars().rev().collect::<String>());
    // TODO: Move out directories to parent (allows to send in mocked list during tests)
    let mut directories: VecDeque<String> = VecDeque::new();
    directories.push_back(String::from(config.scan_root.as_str()));

    let arc_directories = Arc::new(Mutex::new(directories));
    let mut handles = vec![];
    let mut receivers = vec![];

    for _ in 0..config.num_threads {
        let arc_dirs = Arc::clone(&arc_directories);
        let pattern = Arc::clone(&pattern);
        let config = config.clone();
        let (tx, rs) = channel();
        receivers.push(rs);

        let handle = thread::spawn(move || {
            let mut best_s = 0;
            let mut best_res = String::from("");
            loop {
                let mut dirs = arc_dirs.lock().unwrap();
                let dir: ReadDir = match dirs.pop_front() {
                    Some(path_str) => {
                        drop(dirs);
                        let current_path: &Path = Path::new(path_str.as_str());
                        fs::read_dir(current_path).unwrap()
                    }
                    None => {
                        tx.send((best_s, best_res)).unwrap();
                        break;
                    }
                };

                for thing in dir {
                    let path: PathBuf = match thing {
                        Ok(de) => de.path(),
                        Err(_) => break,
                    };
                    if !path.is_dir() {
                        continue;
                    };

                    let path_str = path.to_str().unwrap();
                    let path_string: String = String::from(path_str);

                    if path_string.contains("/.") {
                        continue;
                    };

                    let mut parts: Vec<&str> = path_str.split('/').collect();
                    let folder: &str = parts.pop().unwrap();

                    if config.ignores.contains(folder) {
                        continue;
                    }

                    let rev_line = path_str.chars().rev().collect::<String>();

                    let score = match fuzzy_match(&rev_line, &pattern) {
                        Some(s) => s,
                        None => 0,
                    };

                    if score > best_s {
                        best_s = score;
                        best_res = path_string;
                    }

                    let mut dirs = arc_dirs.lock().unwrap();
                    dirs.push_back(String::from(path_str));
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut best_score = 0;
    let mut best_result = String::from("");
    for rs in receivers {
        let (best_s, best_res) = rs.recv().unwrap();
        if best_s > best_score {
            best_score = best_s;
            best_result = best_res;
        }
    }

    if best_score < 10 {
        best_result = String::from(".");
    }

    best_result.replace(' ', "\\ ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config;
    use std::env;

    macro_rules! vec_string {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    fn create_test_folders(folders: Vec<String>) -> (Config, PathBuf) {
        let mut config: Config = test_config();
        let mut dir = env::temp_dir();
        dir.push("fj_matcher_tests");
        config.scan_root = String::from(dir.as_path().to_str().unwrap());

        for folder in folders {
            let mut d = dir.clone();
            d.push(folder);

            fs::create_dir_all(d.as_path()).unwrap();
        }

        (config, dir)
    }

    #[test]
    fn test_basic_exact_match() {
        let (config, mut dir) = create_test_folders(vec_string!["test"]);

        let result: String = matcher(config, String::from("test"));
        dir.push("test");
        assert_eq!(result, String::from(dir.as_path().to_str().unwrap()));
    }

    #[test]
    fn test_prefer_later_in_string() {
        let lines: Vec<String> = vec_string!["projects", "projects/project", "projects/hello"];
        let (config, mut dir) = create_test_folders(lines);
        let result: String = matcher(config, String::from("project"));
        dir.push("projects/project");
        assert_eq!(result, String::from(dir.as_path().to_str().unwrap()));
    }

    #[test]
    fn test_handles_space_in_path() {
        let lines: Vec<String> =
            vec_string!["projects", "projects/project other", "projects/hello"];
        let (config, mut dir) = create_test_folders(lines);
        dir.push("projects/project other");

        let result: String = matcher(config, String::from("other"));
        assert!(result.as_str().ends_with("/projects/project\\ other"));
    }
}

#[cfg(all(feature = "nightly", test))]
mod benchs {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::env;
    use std::fs::{File, OpenOptions};
    use std::io::prelude::*;
    use test::{black_box, Bencher};

    fn get_rand_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .collect::<String>()
    }

    #[bench]
    fn bench_scan_random_strings(b: &mut Bencher) {
        let mut lines: Vec<String> = Vec::new();
        (0..1000).for_each(|_x| {
            lines.push(get_rand_string(1000));
        });
        let mut dir = env::temp_dir();
        dir.push("bench_random_string");
        let file_name: &str = dir.to_str().unwrap();
        let mut file: File = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(file_name)
            .unwrap();

        for line in lines {
            file.write(line.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
        }

        b.iter(|| {
            let file: File = File::open(file_name).unwrap();
            let reader: BufReader<File> = BufReader::new(file);
            black_box(matcher(default_config(), get_rand_string(20)));
        });
    }
}
