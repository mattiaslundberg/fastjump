#![cfg_attr(all(feature = "nightly", test), feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

mod config;
mod fj_matcher;
mod save;
use config::{get_config_pb, Config};
use save::save;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Cli {
    pattern: String,

    #[structopt(long = "--config", parse(from_os_str))]
    config_file: Option<PathBuf>,

    #[structopt(short, long = "--save-visit")]
    save_visit: bool,
}

fn main() {
    let args: Cli = Cli::from_args();
    let config_file = args.config_file;
    let config = get_config_pb(config_file);

    if args.save_visit {
        save(config, args.pattern);
        return;
    }
    change(config, args.pattern);
}

fn change(config: Config, pattern: String) -> String {
    let best_result: String = fj_matcher::matcher(config.clone(), pattern);

    save(config, best_result.clone());
    println!("{}", best_result);
    best_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::test_config;

    #[test]
    fn test_good_match() {
        let pattern = String::from("empty");
        assert_eq!(change(test_config(), pattern), "test_configs/empty")
    }

    #[test]
    fn test_no_match() {
        let pattern = String::from("nonexisting");
        assert_eq!(change(test_config(), pattern), ".")
    }
}
