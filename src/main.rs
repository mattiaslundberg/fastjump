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
    let config = get_config_file();

    if args.scan {
        scan(config, args.pattern);
    } else {
        change(config, args.pattern);
    }
}

fn get_config_file() -> String {
    match std::env::var("FASTJUMP_CONFIG") {
        Ok(val) => val,
        Err(_e) => String::from("~/.fastjump"),
    }
}

fn scan(config: String, pattern: String) {
    println!("Scanning {}", pattern)
}

fn change(config: String, pattern: String) {
    println!("Searching for {}", pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_file() {
        assert_eq!(get_config_file(), "~/.fastjump");
        std::env::set_var("FASTJUMP_CONFIG", "/tmp/fastjump_test");
        assert_eq!(get_config_file(), "/tmp/fastjump_test");
    }
}
