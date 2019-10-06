extern crate yaml_rust;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use yaml_rust::{yaml, YamlLoader};

pub struct Config {
    pub ignores: HashSet<String>,
    pub scan_root: String,
    pub num_threads: u8,
}

fn get_default_config_file() -> String {
    let home = std::env::var("HOME").unwrap();
    format!("{}/.fastjump.yml", home)
}

pub fn default_config() -> Config {
    let scan_root = std::env::var("HOME").unwrap();
    Config {
        ignores: HashSet::new(),
        scan_root,
        num_threads: 1,
    }
}

#[cfg(test)]
pub fn test_config() -> Config {
    let ignores = HashSet::new();
    Config {
        ignores,
        scan_root: String::from("test_configs"),
        num_threads: 1,
    }
}

fn read_config_from_file(mut file: File) -> Config {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let datas = YamlLoader::load_from_str(contents.as_str()).unwrap();
    let data = &datas[0];

    let mut ignores = HashSet::new();
    let default_ignores = yaml::Array::new();
    let ignore_data = data["ignores"].as_vec().unwrap_or(&default_ignores);

    for d in ignore_data {
        ignores.insert(String::from(d.as_str().unwrap()));
    }

    let default_root = ".";
    let scan_root = data["scan_root"].as_str().unwrap_or(&default_root);

    let num_threads: u8 = data["num_threads"]
        .as_i64()
        .unwrap_or(3)
        .try_into()
        .unwrap();

    Config {
        ignores,
        scan_root: String::from(scan_root),
        num_threads,
    }
}

pub fn get_config(maybe_config_file: Option<&Path>) -> Config {
    let default_path = get_default_config_file();
    let config_file = match maybe_config_file {
        Some(f) => f,
        None => Path::new(&default_path),
    };

    match File::open(config_file) {
        Ok(f) => read_config_from_file(f),
        Err(_) => default_config(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_config_file_existing() {
        let config = get_config(Some(Path::new("nonexisting")));
        assert_eq!(config.ignores, HashSet::new());
    }

    #[test]
    fn test_parse_file() {
        let config = get_config(Some(Path::new("test_configs/simple.yml")));
        let mut expected = HashSet::new();
        expected.insert(String::from("node_modules"));
        assert_eq!(config.ignores, expected);
        assert_eq!(config.scan_root, String::from("test_configs"));
        assert_eq!(config.num_threads, 5);
    }

    #[test]
    fn test_missing_ignores() {
        let config = get_config(Some(Path::new("test_configs/missing_ignores.yml")));
        assert_eq!(config.ignores, HashSet::new());
        assert_eq!(config.scan_root, String::from("test_configs"));
    }

    #[test]
    fn test_missing_scan_root() {
        let config = get_config(Some(Path::new("test_configs/missing_root.yml")));
        assert_eq!(config.ignores, HashSet::new());
        assert_eq!(config.scan_root, String::from("."));
    }

    #[test]
    fn test_missing_threads() {
        let config = get_config(Some(Path::new("test_configs/missing_root.yml")));
        assert_eq!(config.num_threads, 3);
    }

    #[test]
    #[should_panic(expected = "TryFromIntError")]
    fn test_threads_too_large() {
        get_config(Some(Path::new("test_configs/large_threads.yml")));
    }
}
