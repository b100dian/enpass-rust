mod model;

use anyhow::Result;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use model::vault::Vault;
use model::vaultcommand::VaultCommand;
use std::io;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    vault: std::path::PathBuf,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List the items in the vault
    List {},
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Some(Commands::List {}) => {}
        None => {
            println!("{}", Cli::command().render_usage());
            std::process::exit(1);
        }
    };

    let vault = Vault::new(args.vault)?;

    let mut pass = String::new();
    println!("Enter vault password:");
    io::stdin().read_line(&mut pass)?;

    let connection = vault.login(pass.trim_end().as_bytes())?;

    let vault_connection = VaultCommand::new(connection);
    let items = vault_connection.list()?;
    for item in items {
        println!("{}", item);
    }

    Ok(())
}
