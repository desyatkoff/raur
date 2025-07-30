#!/usr/bin/env bash

#################################
#                               #
#   ____      _   _   _ ____    #
#  |  _ \    / \ | | | |  _ \   #
#  | |_) |  / _ \| | | | |_) |  #
#  |  _ <  / ___ \ |_| |  _ <   #
#  |_| \_\/_/   \_\___/|_| \_\  #
#                               #
#################################


# 0. Pre-installation preparations

set -euo pipefail

IFS=$'\n\t'

echo "Welcome to RAUR installer script"

read -rp "Continue? [Y/n] " confirm

[[ -z "$confirm" || "$confirm" =~ ^[Yy]$ ]] || exit 1


# 1. Check if Rust is installed

echo "Checking if Rust is installed..."

if ! command -v rustup &> /dev/null; then
    echo "Rust is not installed. Installing Rust... (needed to compile RAUR)"

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    export PATH="$PATH:$HOME/.cargo/bin"

    source "$HOME/.cargo/env"

    echo "Rust has been installed"
else
    echo "Rust is already installed"
fi


# 2. Check if running in the project directory

if [ ! -f "Cargo.toml" ]; then
    echo "This script must be run from the project root"

    exit 1
fi


# 3. Clean files (in case if already installed)

echo "Cleaning old project files..."

cargo clean || true

[ -f /usr/bin/raur ] && sudo rm -vf /usr/bin/raur || true


# 4. Compile the Rust project

echo "Compiling RAUR..."

cargo build --release


# 5. Copy compiled binary to the `/usr/bin/` directory

echo "Copying binary file to '/usr/bin/'..."

sudo cp -v \
    ./target/release/raur \
    /usr/bin/


# 6. After installation

echo "RAUR installed successfully"


# Success!
# Enjoy your new *blazingly fast* Rusty Arch User Repository helper
