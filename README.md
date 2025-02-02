# cargo-verset

[![Crates.io](https://img.shields.io/crates/v/cargo-verset.svg)](https://crates.io/crates/cargo-verset)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/cargo-verset/blob/main/LICENSE)
[![CI](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml)

`cargo-verset` is a tool to change the version in your Cargo.toml file.

## Installation

```sh
cargo install cargo-verset
```

or with `cargo binstall`:

```sh
cargo binstall cargo-verset
```

## Usage

```sh
Usage: cargo-verset.exe [OPTIONS] --ver <VER>

Options:
  -v, --ver <VER>    Version to set in the workspace
  -p, --path <PATH>  Path to the directory containing the Cargo.toml file
  -d, --dry-run      Run the program without making any changes
  -h, --help         Print help
  -V, --version      Print version
```
