use clap::CommandFactory;
use clap::{Parser, Subcommand};
use enpass::lite::vault::Vault;
use enpass::lite::vaultcommand::VaultCommand;

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
    Dump {
        item_id: u32,
    },
}

fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("info")?.start()?;
    let args = Cli::parse();

    match &args.command {
        None => {
            eprintln!("{}", Cli::command().render_usage());
            std::process::exit(1);
        }
        _ => {}
    };

    let mut vault = Vault::new(args.vault)?;

    let pass = rpassword::prompt_password("Enter vault password:")?;

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
        Some(Commands::Dump { item_id }) => {
            let dump = vault_command.dump(item_id)?;
            for kv in dump {
                println!("{}\t{}", kv.key, kv.value);
            }
        }
    }

    Ok(())
}
