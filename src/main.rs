mod model;

use anyhow::{Error, Result};
use clap::Parser;
use model::vault::{Vault, VaultConnection};
use std::io;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    vault: std::path::PathBuf,

    command: String,
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    println!("Vault: {:?}, Command:{:?}", args.vault, args.command);

    let vault = Vault::new(args.vault)?;
    println!("{:x?}", vault.salt());

    let mut pass = String::new();
    let count = io::stdin().read_line(&mut pass)?;
    println!("Read {} bytes", count);

    let c = vault.login(pass.as_bytes())?;
    println!("I'm in");

    let _vault_connection = VaultConnection::new(c);
    Ok(())
}
