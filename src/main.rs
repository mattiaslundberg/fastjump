#![cfg_attr(all(feature = "nightly", test), feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

mod config;
mod fj_matcher;
use config::{get_config, Config};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Cli {
    pattern: String,
}

fn main() {
    let args: Cli = Cli::from_args();
    let config: Config = get_config(None);

    change(config, args.pattern);
}

fn change(config: Config, pattern: String) -> String {
    let best_result: String = fj_matcher::matcher(config, pattern);

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
