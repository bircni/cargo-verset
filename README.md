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
A cargo plugin to change the version of a package in the Cargo.toml file

Usage: cargo-verset <COMMAND>

Commands:
  package     Set the version of the package in a Cargo.toml file
  dependency  Set the version of a dependency in a Cargo.toml file
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
