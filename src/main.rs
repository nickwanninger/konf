#![warn(rust_2018_idioms)]

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    header: Option<String>,

    #[clap(default_value = "Kconfig")]
    config: String,
}

fn main() {
    let args = Args::parse();
    let config = konf::parser::parse_file(&args.config);

    if let Err(err) = config {
        eprintln!("failed to parse {}: {}", args.config, err);
        return;
    }

    let mut config = config.unwrap();
    config.load_default();
    println!("{}", config);
    config.load(".config").unwrap();
    println!("{}", config);
    config.save_config(".config").unwrap();
}
