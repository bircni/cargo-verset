# cargo-verset

[![Crates.io](https://img.shields.io/crates/v/cargo-verset.svg)](https://crates.io/crates/cargo-verset)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/cargo-verset/blob/main/LICENSE)
[![CI](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml)

`cargo-verset` is a tool to change the version in your Cargo.toml file.

## Usage

```sh
Usage: cargo-verset [OPTIONS] --semver <SEMVER>

Options:
  -s, --semver <SEMVER>  Version
  -p, --path <PATH>      Path to look for the Cargo.toml
  -d, --dry-run          Dry run
  -h, --help             Print help
  -V, --version          Print version

```
