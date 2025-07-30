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

fn get_installed_version(pkg: &str) -> Option<String> {
    let output = Command::new("pacman")
        .args([
            "-Q",
            pkg
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout
        .split_whitespace()
        .collect();

    if parts.len() >= 2 {
        return Some(parts[1].to_string());
    } else {
        return None;
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
            println!("Installing {package}...");

            install_package(package);
        }
        Commands::Update { package } => {
            println!("Updating {package}...");

            update_package(package);
        }
        Commands::Remove { package } => {
            // TODO: Make it actually remove package

            println!("Removing {package}...");
        }
    };
}

fn install_package(pkg: &str) {
    println!("Checking AUR for `{}`...", pkg);

    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg={}", pkg);
    let response = get(&url)
        .expect("Failed to query AUR");
    let aur: AurResponse = response
        .json()
        .expect("Failed to parse AUR JSON");

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
        "--noconfirm",
        "--skippgpcheck"
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
            eprintln!("PGP error! You might need to import the missing GPG key manually");
            eprintln!("Try running: gpg --recv-keys <KEY>");
        }

        std::process::exit(1);
    }
}

fn update_package(pkg: &str) {
    println!("Checking installed version for `{}`...", pkg);

    let installed_version = get_installed_version(pkg);

    if installed_version.is_none() {
        println!("`{}` is not installed. Use `raur install {}` instead", pkg, pkg);

        return;
    }

    let installed_version = installed_version.unwrap();

    println!("Installed version: {}", installed_version);

    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg={}", pkg);
    let response = get(&url)
        .expect("Failed to query AUR");
    let aur: AurResponse = response
        .json()
        .expect("Failed to parse AUR JSON");

    if aur.resultcount == 0 {
        eprintln!("Package `{}` not found in AUR", pkg);

        return;
    }

    let info = &aur.results[0];

    println!("AUR version: {}", info.version);

    if info.version != installed_version {
        println!("Update available! Updating `{}`...", pkg);

        install_package(pkg);
    } else {
        println!("`{}` is already up to date!", pkg);
    }
}
