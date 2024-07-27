mod model;

use clap::{Error, Parser};
use model::vault::Vault;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    vault: std::path::PathBuf,

    command: String,
}

fn main() -> std::result::Result<(), std::io::Error> {
    let args = Cli::parse();

    println!("Vault: {:?}, Command:{:?}", args.vault, args.command);

    let vault = Vault::new(args.vault)?;
    Ok(println!("{:x?}", vault.salt()))
}
