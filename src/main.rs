use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Cli {
    #[structopt(short, long)]
    scan: bool,

    pattern: String,
}

fn main() {
    let cli = Cli::from_args();

    println!("{:#?}", cli);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        assert_eq!(1, 1)
    }
}
