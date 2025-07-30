use clap::{
    Parser,
    Subcommand,
};

#[derive(Parser)]
#[command(name = "raur")]
#[command(version = "0.1.0")]
#[command(about = "RAUR - Rusty AUR helper", long_about = "RAUR is an Arch User Repository helper for managing AUR packages with ease")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        package: String,
    },
    Update {
        package: String,
    },
    Remove {
        package: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
            // TODO: Make it actually install package

            println!("Installing {package}...");
        }
        Commands::Update { package } => {
            // TODO: Make it actually update package

            println!("Updating {package}...");
        }
        Commands::Remove { package } => {
            // TODO: Make it actually remove package

            println!("Removing {package}...");
        }
    }
}
