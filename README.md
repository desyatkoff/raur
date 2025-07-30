# RAUR

```
 ____      _   _   _ ____  
|  _ \    / \ | | | |  _ \ 
| |_) |  / _ \| | | | |_) |
|  _ <  / ___ \ |_| |  _ < 
|_| \_\/_/   \_\___/|_| \_\ 
```

## Description

RAUR is an Arch User Repository helper for managing AUR packages with ease

## Table of Contents

1. [RAUR](#raur)
2. [Description](#description)
3. [Table of Contents](#table-of-contents)
4. [Features](#features)
5. [Installation](#installation)
6. [Usage](#usage)
7. [Feedback](#feedback)

## Features

* Fast
* Minimal

## Installation

1. Clone the repository
    ```Shell
    git clone https://github.com/desyatkoff/raur.git
    ```
2. Go to the repository directory
    ```Shell
    cd raur/
    ```
3. Choose your installation way
    * Auto
        1. Run installer script
            ```Shell
            sh install.sh
            ```
    * Manual
        1. Compile the Rust project
            ```Shell
            cargo build --release
            ```
        2. Copy compiled binary to the `/usr/bin/` directory
            ```Shell
            sudo cp ./target/release/raur /usr/bin/
            ```

## Usage

* Get help
    + Short
        ```Shell
        raur -h
        ```
    + Full
        ```Shell
        raur --help
        ```
* Install package
    ```Shell
    raur install <package>
    ```
* Update package
    ```Shell
    raur update <package>
    ```
* Uninstall/Remove package
    ```Shell
    raur remove <package>
    ```
* Skip PGP check (not recommended)
    ```Shell
    raur --skip-pgp-check install <package>
    ```
* Get version
    + Short
        ```Shell
        raur -V
        ```
    + Full
        ```Shell
        raur --version
        ```

## Feedback  

Found a bug? [Open an issue](https://github.com/desyatkoff/raur/issues/new)
