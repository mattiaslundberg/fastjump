use fuzzy_matcher::skim::fuzzy_match;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn matcher(reader: BufReader<File>, pattern: String) -> String {
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

    best_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_good_match() {}
}
