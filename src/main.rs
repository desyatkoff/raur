use clap::{
    Parser,
    Subcommand,
};
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use reqwest::blocking::get;
use serde::Deserialize;
use tempfile::tempdir;

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

#[derive(Deserialize)]
struct AurResponse {
    resultcount: u32,
    results: Vec<PackageInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PackageInfo {
    name: String,
    version: String,
    description: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
            install_package(package);
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

fn install_package(pkg: &str) {
    println!("Checking AUR for `{}`...", pkg);

    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg={}", pkg);
    let result = get(&url)
        .expect("Failed to send request");
    let aur: AurResponse = result
        .json()
        .expect("Failed to parse JSON");

    if aur.resultcount == 0 {
        eprintln!("Package `{}` not found in the AUR.", pkg);

        return;
    }

    let info = &aur.results[0];

    println!(
        "Found `{}` (v{}): {}",
        info.name,
        info.version,
        info.description
            .as_deref()
            .unwrap_or("No description")
    );

    let temp_dir = tempdir()
        .expect("Failed to create temp directory");
    let repo_path = temp_dir
        .path()
        .join(&info.name);

    println!("Cloning into temp directory: {}", repo_path.display());

    let status = Command::new("git")
        .args([
            "clone",
            &format!(
                "https://aur.archlinux.org/{}.git",
                info.name
            )
        ])
        .arg(&repo_path)
        .status()
        .expect("Failed to run `git`");

    if !status.success() {
        eprintln!("`git clone` failed");

        return;
    }

    println!("Running `makepkg -si`...");

    let mut makepkg = Command::new("makepkg");

    makepkg.args([
        "-si",
        "--noconfirm"
    ]);
    makepkg.current_dir(&repo_path);

    let output = makepkg
        .output()
        .expect("Failed to run `makepkg`");

    if output.status.success() {
        println!("Done installing `{}`!", info.name);
    } else {
        eprintln!("`makepkg` failed for `{}`", info.name);

        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("One or more PGP signatures could not be verified") {
            eprintln!("    PGP error! You might need to import the missing GPG key manually");
            eprintln!("    Try running: gpg --recv-keys <KEY>");
        }

        std::process::exit(1);
    }
}
