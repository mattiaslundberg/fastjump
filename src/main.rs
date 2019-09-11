mod fj_matcher;
mod scan;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
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
        scan::scan(config, args.pattern);
    } else {
        change(config, args.pattern);
    }
}

fn get_config_file() -> String {
    let home = std::env::var("HOME").unwrap();
    match std::env::var("FASTJUMP_CONFIG") {
        Ok(val) => val,
        Err(_e) => String::from(format!("{}/.fastjump", home)),
    }
}

fn change(config: &Path, pattern: String) -> String {
    let file = match File::open(config) {
        Ok(f) => f,
        Err(e) => panic!("Could not open file {}", e),
    };

    let reader = BufReader::new(file);
    let best_result: String = fj_matcher::matcher(reader, pattern);

    println!("{}", best_result);
    best_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_file() {
        assert!(get_config_file().ends_with(".fastjump"));
        std::env::set_var("FASTJUMP_CONFIG", "/tmp/fastjump_test");
        assert_eq!(get_config_file(), "/tmp/fastjump_test");
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
