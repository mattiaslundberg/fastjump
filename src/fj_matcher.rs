use fuzzy_matcher::skim::fuzzy_match;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, SeekFrom};

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
    use std::fs::OpenOptions;
    use std::io::prelude::*;

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

    #[test]
    fn test_basic_exact_match() {
        let lines: Vec<String> = vec_string!["/test", "/other"];
        let reader: BufReader<File> = get_reader("basic_exact", lines);
        let result: String = matcher(reader, String::from("test"));
        assert_eq!(result, String::from("/test"));
    }

    #[test]
    fn test_prefer_later_in_string() {
        let lines: Vec<String> =
            vec_string!["/projects/", "/projects/project/", "/projects/hello/"];
        let reader: BufReader<File> = get_reader("prefer_later", lines);
        let result: String = matcher(reader, String::from("project"));
        assert_eq!(result, String::from("/projects/project/"));
    }
}
