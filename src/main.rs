use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "auria-cli",
    version,
    about = "Auria CLI - Command Line Interface for Auria Node"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Start,
    Stop,
    Status,
    Wallet {
        #[arg(long)]
        address: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Start => {
            println!("Starting Auria Node...");
            println!("Use 'auria start' command instead");
        }
        Command::Stop => {
            println!("Stopping Auria Node...");
        }
        Command::Status => {
            println!("Auria CLI - Use 'auria status' for node status");
        }
        Command::Wallet { address } => {
            if let Some(addr) = address {
                println!("Wallet address: {}", addr);
            } else {
                println!("No wallet connected");
            }
        }
    }

    Ok(())
}
