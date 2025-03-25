# DigiDownload CLI

A simple CLI tool to download books from digi4school.at, written in Rust.

Uses the [DigiDownload Core](https://github.com/digiDownload/digiDownload) library.

## Getting started
### Installation
#### Prerequisites
- Ensure you have [Rust installed](https://rustup.rs/).

#### Install CLI
```bash
cargo install --git https://github.com/digiDownload/cli
```

### Usage
![](resources/usage.gif)

1. Open the CLI by running `digidownload` in your terminal.
2. Enter your email and password to log in.
3. Select the books you want to download.
4. If a book has multiple volumes, choose the volume you need.
5. After downloading, you can open the output folder by entering `y` when prompted.

## Features

- Downloads are done asynchronously for faster speeds.
- Caches login email so you don't have to enter it every time.
- Allows you to interactively choose books and volumes.

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request on [Github](https://github.com/digiDownload/cli).
