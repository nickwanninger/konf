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


// konf [flags] <Kconfig>

fn main() {
    let args = Args::parse();
    println!("Config location: {}", args.config);


    let file_text = std::fs::read_to_string(&args.config);
    if let Err(e) = file_text {
        panic!("Failed to read: {}", e);
    }
    let config = konf::parser::parse(&file_text.unwrap());

    if let Err(_e) = config {
        eprintln!("failed to parse {}", args.config);
        return;
    }
}
