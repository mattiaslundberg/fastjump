extern crate test;
use fuzzy_matcher::skim::fuzzy_match;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn matcher(reader: BufReader<File>, pattern: String) -> String {
    let mut best_score = 0;
    let mut best_result: String = String::new();
    let pattern = pattern.chars().rev().collect::<String>();

    for line in reader.lines() {
        let line = line.unwrap().chars().rev().collect::<String>();

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

    best_result.chars().rev().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::env;
    use std::fs::OpenOptions;
    use std::io::prelude::*;
    use std::io::SeekFrom;
    use test::{black_box, Bencher};

    macro_rules! vec_string {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    fn get_reader(file_name: &str, lines: Vec<String>) -> BufReader<File> {
        let mut dir = env::temp_dir();
        dir.push(file_name);
        let mut file: File = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(dir)
            .unwrap();

        for line in lines {
            file.write(line.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
        }

        file.seek(SeekFrom::Start(0)).unwrap();
        BufReader::new(file)
    }

    fn get_rand_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .collect::<String>()
    }

    #[test]
    fn test_basic_exact_match() {
        let lines: Vec<String> = vec_string!["/test", "/other"];
        let reader: BufReader<File> = get_reader("basic_exact", lines);
        let result: String = matcher(reader, String::from("test"));
        assert_eq!(result, String::from("/test"));
    }

    #[test]
    fn test_prefer_later_in_string() {
        let lines: Vec<String> = vec_string!["/projects", "/projects/project", "/projects/hello"];
        let reader: BufReader<File> = get_reader("prefer_later", lines);
        let result: String = matcher(reader, String::from("project"));
        assert_eq!(result, String::from("/projects/project"));
    }

    #[bench]
    fn bench_scan_random_strings(b: &mut Bencher) {
        let mut lines: Vec<String> = Vec::new();
        (0..1000).for_each(|_x| {
            lines.push(get_rand_string(100));
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
            black_box(matcher(reader, get_rand_string(20)));
        });
    }

}
