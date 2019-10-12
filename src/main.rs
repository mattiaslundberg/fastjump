#![cfg_attr(all(feature = "nightly", test), feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

mod cache;
mod config;
mod fj_matcher;
use cache::save;
use config::{get_config_pb, Config};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "fastjump")]
/// Simple way to navigate between directories
///
/// See https://github.com/mattiaslundberg/fastjump#configure for information on how to configure fastjump before first use.
struct Cli {
    /// If passing `--save-visit` the location to save, otherwise will be used to change directories
    ///
    /// See help for `--save-visit` for more information how pattern is used in that case.
    /// Otherwise fastjump will attempt to match all existing directories from `scan_root` (if specified in config) or `HOME` if not configured. All directories will be fuzzy matched against the pattern and the best option will be printed in a way that it can be directly used by `cd`, `cd $(fastjump <some pattern>)`. If no good match is found it will print `.` and `cd` will change to the current directory.
    pattern: String,

    #[structopt(long = "--config", parse(from_os_str))]
    /// Use a non standard configuration file, default: `~/.fastjump.yml`
    ///
    /// The file must exist or default config with no ignores and scan_root `~` will be used.
    /// See https://github.com/mattiaslundberg/fastjump for avaliable configuration options.
    config_file: Option<PathBuf>,

    #[structopt(short, long = "--save-visit")]
    /// Requires config. Save location in pattern to cache file
    ///
    /// Saves the location in pattern to configured cache file. See help for `--config` for how to configure.
    /// Will update the cache file and give the saved location a better match when matching.
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
