#[cfg(test)]
use crate::config::test_config;
use crate::config::Config;
use fuzzy_matcher::skim::fuzzy_match;
#[cfg(test)]
use rand::distributions::Alphanumeric;
#[cfg(test)]
use rand::Rng;
use std::collections::VecDeque;
#[cfg(test)]
use std::env;
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn matcher(config: Config, pattern: String) -> String {
    let pattern = Arc::new(pattern.chars().rev().collect::<String>());
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
fn get_rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect::<String>()
}

#[cfg(test)]
fn create_test_folders(folders: Vec<String>) -> (Config, PathBuf) {
    let mut config: Config = test_config();
    let mut dir = env::temp_dir();
    let mut tmp_dir = get_rand_string(3);
    tmp_dir.push_str("_fj_matcher_tests");
    dir.push(tmp_dir);

    match fs::remove_dir_all(dir.clone()) {
        Ok(()) => (),
        Err(e) => {
            println!("Failed to remove directory {}", e);
            ()
        }
    };

    config.scan_root = String::from(dir.as_path().to_str().unwrap());

    for folder in folders {
        let mut d = dir.clone();
        d.push(folder);

        fs::create_dir_all(d.as_path()).unwrap();
    }

    (config, dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_string {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
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
    use test::{black_box, Bencher};

    fn generate_lines() -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        (0..1000).for_each(|_x| {
            let mut s: String = get_rand_string(700);
            for i in 1..6 {
                s.insert(100 * i, '/');
            }
            lines.push(s);
        });
        lines
    }

    #[bench]
    fn bench_scan_random_strings_single_thread(b: &mut Bencher) {
        let lines = generate_lines();
        let (mut config, _dir) = create_test_folders(lines);
        config.num_threads = 1;

        b.iter(|| {
            black_box(matcher(config.clone(), get_rand_string(20)));
        });
    }

    #[bench]
    fn bench_scan_random_strings_two_threads(b: &mut Bencher) {
        let lines = generate_lines();
        let (mut config, _dir) = create_test_folders(lines);
        config.num_threads = 2;

        b.iter(|| {
            black_box(matcher(config.clone(), get_rand_string(20)));
        });
    }

    #[bench]
    fn bench_scan_random_strings_five_threads(b: &mut Bencher) {
        let lines = generate_lines();
        let (mut config, _dir) = create_test_folders(lines);
        config.num_threads = 5;

        b.iter(|| {
            black_box(matcher(config.clone(), get_rand_string(20)));
        });
    }

    #[bench]
    fn bench_scan_random_strings_ten_threads(b: &mut Bencher) {
        let lines = generate_lines();
        let (mut config, _dir) = create_test_folders(lines);
        config.num_threads = 10;

        b.iter(|| {
            black_box(matcher(config.clone(), get_rand_string(20)));
        });
    }
}
