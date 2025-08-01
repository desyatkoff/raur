/*
 * This file is part of RAUR
 *
 * Copyright (C) 2025 Desyatkov Sergey
 *
 * RAUR is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License,
 * or (at your option) any later version
 *
 * RAUR is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details
 *
 * You should have received a copy of the GNU General Public License
 * along with RAUR. If not, see <https://www.gnu.org/licenses/>
 */

use clap::{
    Parser,
    Subcommand,
};
use std::process::Command;
use reqwest::blocking::get;
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Parser)]
#[command(name = "raur")]
#[command(version = "1.0.1")]
#[command(about = "RAUR is an Arch User Repository helper for managing AUR packages with ease")]
struct Cli {
    #[arg(long)]
    skip_pgp_check: bool,
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

            install_package(package, cli.skip_pgp_check);
        }
        Commands::Update { package } => {
            println!("Updating {package}...");

            update_package(package, cli.skip_pgp_check);
        }
        Commands::Remove { package } => {
            println!("Removing {package}...");

            remove_package(package);
        }
    };
}

fn install_package(pkg: &str, skip_pgp: bool) {
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
        "--noconfirm"
    ]);

    if skip_pgp {
        makepkg.arg("--skippgpcheck");
    }

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
            eprintln!("PGP error! You might need to import the missing GPG key manually or skip PGP check");

            if let Some(line) = stderr.lines().find(|l| l.contains("key") && l.contains("unknown")) {
                for word in line.split_whitespace() {
                    if word.len() >= 16 && word.chars().all(|c| c.is_ascii_hexdigit()) {
                        eprintln!(
                            "Try running:\n    A. `gpg --recv-keys {}`\n    B. `raur install --skip-pgp-check`",
                            word
                        );

                        return;
                    }

                    if word.len() >= 17 && word.ends_with(')') {
                        let trimmed = word.trim_end_matches(')');

                        if trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                            eprintln!(
                                "Try running:\n    A. `gpg --recv-keys {}`\n    B. `raur install --skip-pgp-check`",
                                trimmed
                            );

                            return;
                        }
                    }
                }
            }

            eprintln!("Try running:\n    A. `gpg --recv-keys <KEY>`\n    B. `raur install --skip-pgp-check`");
        }

        std::process::exit(1);
    }
}

fn update_package(pkg: &str, skip_pgp: bool) {
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

        install_package(pkg, skip_pgp);
    } else {
        println!("`{}` is already up to date!", pkg);
    }
}

fn remove_package(pkg: &str) {
    let status = Command::new("sudo")
        .arg("pacman")
        .args(&[
            "-Rns",
            pkg
        ])
        .status()
        .expect("Failed to run `pacman`");

    if status.success() {
        println!("Package `{}` removed successfully!", pkg);
    } else {
        eprintln!("Failed to remove package `{}`", pkg);
    }
}
