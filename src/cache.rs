use crate::config::Config;
use linked_hash_map::LinkedHashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use yaml_rust::{yaml, Yaml, YamlEmitter};

pub fn read_current_state(previous_visits: PathBuf, yaml_string: &mut String) {
    let mut previous_visits_dir = previous_visits.clone();
    previous_visits_dir.pop();
    let _ = create_dir_all(previous_visits_dir.as_path());

    let maybe_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(previous_visits.clone());
    match maybe_file {
        Ok(mut file) => file.read_to_string(yaml_string).unwrap(),
        Err(e) => {
            println!(
                "Error: Failed to read state {}: {}",
                previous_visits.to_str().unwrap(),
                e
            );
            0
        }
    };
}

pub fn write_new_state(previous_visits: PathBuf, location: String, data_hash: &mut yaml::Hash) {
    let key = Yaml::String(location);
    let previous_value = data_hash
        .entry(key.clone())
        .or_insert(Yaml::Integer(0))
        .as_i64()
        .unwrap();
    data_hash.insert(key, Yaml::Integer(previous_value + 1));

    let mut writer = String::new();
    let mut emitter = YamlEmitter::new(&mut writer);
    emitter.dump(&Yaml::Hash(data_hash.clone())).unwrap();
    let f = OpenOptions::new().write(true).open(previous_visits.clone());
    match f {
        Ok(mut f) => f.write_all(writer.as_bytes()).unwrap(),
        Err(e) => {
            println!(
                "Error: Failed to write state {}: {}",
                previous_visits.to_str().unwrap(),
                e
            );
        }
    }
}

fn read_current_state_to_yamlmap(config: Config) -> yaml::Hash {
    let default = yaml::Hash::new();
    if config.previous_visits.is_none() {
        return default;
    }
    let previous_visits = config.previous_visits.unwrap();
    let mut yaml_string = String::new();
    read_current_state(previous_visits, &mut yaml_string);

    let datas = yaml::YamlLoader::load_from_str(&yaml_string).unwrap();
    let default = Yaml::Hash(yaml::Hash::new());
    let data = if datas.is_empty() {
        &default
    } else {
        &datas[0]
    };
    data.clone().into_hash().unwrap()
}

pub fn get_current_state(config: Config) -> LinkedHashMap<String, i64> {
    let mut res: LinkedHashMap<String, i64> = LinkedHashMap::new();
    let data = read_current_state_to_yamlmap(config);

    for (key, value) in data {
        let k = key.into_string().unwrap();
        let v = value.into_i64().unwrap();
        res.insert(k, v);
    }

    res
}

pub fn save(config: Config, location: String) {
    let previous_visits = match config.clone().previous_visits {
        None => return,
        Some(p) => p,
    };
    let mut data_hash = read_current_state_to_yamlmap(config);
    write_new_state(previous_visits, location, &mut data_hash);
}

#[cfg(test)]
pub fn write_yaml(path: PathBuf, contents: &[u8]) {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    f.write_all(contents).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config;
    use std::env;
    use std::fs;

    fn setup(filename: &str) -> (Config, PathBuf) {
        let mut config: Config = test_config();
        let mut dir = env::temp_dir();
        dir.push(filename);
        fs::remove_file(dir.clone()).unwrap_or(());
        config.previous_visits = Some(dir.clone());
        (config, dir)
    }

    #[test]
    fn test_get_handles_file_is_unwritable() {
        let mut config: Config = test_config();
        let mut path = PathBuf::new();
        path.push("/unwritable/test.yml");
        config.previous_visits = Some(path);

        let res = get_current_state(config);
        let expected: LinkedHashMap<String, i64> = LinkedHashMap::new();
        assert_eq!(res, expected);
    }

    #[test]
    fn test_save_handles_file_is_unwritable() {
        let mut config: Config = test_config();
        let mut path = PathBuf::new();
        path.push("/unwritable/test.yml");
        config.previous_visits = Some(path);

        let location: String = String::from("something");
        save(config, location);
    }

    #[test]
    fn test_get_handles_file_is_none() {
        let mut config: Config = test_config();
        config.previous_visits = None;
        let res = get_current_state(config);
        let expected: LinkedHashMap<String, i64> = LinkedHashMap::new();
        assert_eq!(res, expected);
    }

    #[test]
    fn test_get_handles_file_is_empty() {
        let mut config: Config = test_config();
        let mut dir = env::temp_dir();
        dir.push("some_file.yaml");
        write_yaml(dir.clone(), b"");

        config.previous_visits = Some(dir);
        let res = get_current_state(config);
        let expected: LinkedHashMap<String, i64> = LinkedHashMap::new();
        assert_eq!(res, expected);
    }

    #[test]
    fn test_returns_from_file() {
        let (config, _) = setup("test_returns_from_file.yml");

        write_yaml(
            config.clone().previous_visits.unwrap(),
            b"---\nsomething: 3",
        );

        let res = get_current_state(config);
        let mut expected: LinkedHashMap<String, i64> = LinkedHashMap::new();
        expected.insert(String::from("something"), 3);
        assert_eq!(res, expected);
    }

    #[test]
    fn test_save_handles_file_is_none() {
        let mut config: Config = test_config();
        config.previous_visits = None;
        let location: String = String::from("something");
        save(config, location);
    }

    #[test]
    fn test_save_creates_file() {
        let (config, dir) = setup("test_creates_file.yml");
        let location: String = String::from("something");
        save(config, location);

        let mut s = String::new();
        read_current_state(dir, &mut s);

        assert_eq!(s, String::from("---\nsomething: 1"));
    }

    #[test]
    fn test_save_creates_folder() {
        let mut config: Config = test_config();
        let mut dir = env::temp_dir();
        dir.push("created_folder");
        let mut file = dir.clone();
        file.push("test_creates_folder.yml");
        fs::remove_file(file.clone()).unwrap_or(());
        fs::remove_dir(dir.clone()).unwrap_or(());
        config.previous_visits = Some(file.clone());
        let location: String = String::from("something");
        save(config, location);

        let mut s = String::new();
        read_current_state(file.clone(), &mut s);

        assert_eq!(s, String::from("---\nsomething: 1"));
    }

    #[test]
    fn test_save_appends_to_file() {
        let (config, dir) = setup("test_appends_to_file.yml");

        write_yaml(
            config.clone().previous_visits.unwrap(),
            b"---\nsomething: 1",
        );

        let location: String = String::from("new");
        save(config, location);

        let mut s = String::new();
        read_current_state(dir, &mut s);

        assert_eq!(s, String::from("---\nsomething: 1\nnew: 1"));
    }

    #[test]
    fn test_save_updates_line_in_file() {
        let (config, dir) = setup("test_updates_line_in_file.yml");

        write_yaml(
            config.clone().previous_visits.unwrap(),
            b"---\nsomething: 1",
        );

        let location: String = String::from("something");
        save(config, location);

        let mut s = String::new();
        read_current_state(dir, &mut s);

        assert_eq!(s, String::from("---\nsomething: 2"));
    }
}
