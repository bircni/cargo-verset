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
Usage: cargo-verset <COMMAND>

Commands:
  package     Set the version of the package in a Cargo.toml file
  dependency  Set the version of a dependency in a Cargo.toml file
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

### Change package version

```sh
cargo verset package --ver 1.2.3
```

Sets the version of the package in the current `Cargo.toml` to `1.2.3`.

With a path to another Cargo.toml:

```sh
cargo verset package --ver 2.0.0 --path ./some/crate
```

Dry run (shows the change without saving):

```sh
cargo verset package --ver 1.2.3 --dry-run
```

### Change dependency version

```sh
cargo verset dependency --name serde --ver 1.0.200
```

Sets the version of the dependency `serde` to `1.0.200` in the current `Cargo.toml`.

With registry and path:

```sh
cargo verset dependency --name serde --ver 1.0.200 --registry crates-io --path ./some/crate
```

Dry run:

```sh
cargo verset dependency --name serde --ver 1.0.200 --dry-run
```
