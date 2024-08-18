use clap::CommandFactory;
use clap::{Parser, Subcommand};
use enpass::lite::vault::Vault;
use enpass::lite::vaultcommand::VaultCommand;
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
    Password {
        item_id: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match &args.command {
        None => {
            println!("{}", Cli::command().render_usage());
            std::process::exit(1);
        }
        _ => {}
    };

    let vault = Vault::new(args.vault)?;

    let mut pass = String::new();
    eprintln!("Enter vault password:");
    io::stdin().read_line(&mut pass)?;

    let connection = vault.login(pass.trim_end().as_bytes())?;
    let vault_command = VaultCommand::new(connection);

    match &args.command {
        Some(Commands::List {}) | None => {
            let items = vault_command.list()?;
            for item in items {
                println!("{}", item);
            }
        }
        Some(Commands::Password { item_id }) => {
            let password = vault_command.password(item_id)?;
            println!("{}", password);
        }
    }

    Ok(())
}
