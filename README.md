# cargo-verset

[![Crates.io](https://img.shields.io/crates/v/cargo-verset.svg)](https://crates.io/crates/cargo-verset)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/cargo-verset/blob/main/LICENSE)
[![CI](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bircni/cargo-verset/actions/workflows/ci.yml)

`cargo-verset` sets package and dependency versions in your `Cargo.toml` files, across a whole workspace and without touching formatting or comments.

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
Usage: cargo verset [OPTIONS] <COMMAND>

Commands:
  package     Set the version of the packages in a crate or workspace
  dependency  Set the version requirement of a dependency in a crate or workspace
  help        Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Print debug output
  -q, --quiet    Only print warnings and errors
  -h, --help     Print help
  -V, --version  Print version
```

Both commands work on the whole workspace the current directory belongs to and share these options:

| Option | Description |
| --- | --- |
| `--manifest-path <PATH>` | `Cargo.toml` (or its directory) of the crate or workspace, defaults to the current directory |
| `-p, --package <NAME>` | Only change these workspace members (repeatable) |
| `--exclude <NAME>` | Do not change these workspace members (repeatable) |
| `-d, --dry-run` | Print what would be changed without writing any file |

Workspace members are resolved through `cargo metadata`, so `cargo` has to be on your `PATH`.

## Examples

### Change package versions

```sh
cargo verset package 1.2.3
```

In a workspace this

- sets `workspace.package.version`,
- sets `package.version` of every member with its own version (members using `version.workspace = true` are left as they are),
- updates the version requirement of path dependencies between the members, including `[workspace.dependencies]`, if the old requirement does not match the new version anymore.

Increment the current version instead of passing one:

```sh
cargo verset package --bump minor
```

Only some members:

```sh
cargo verset package 2.0.0 -p my-core -p my-cli
cargo verset package 2.0.0 --exclude my-experiments
```

### Change dependency versions

```sh
cargo verset dependency serde 1.0.200
```

Sets the requirement of `serde` everywhere it is declared: in `[workspace.dependencies]` and in the `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` and `[target.*]` tables of all members. Entries using `workspace = true` are left as they are, renamed dependencies are found by their `package` name.

Any version requirement is accepted:

```sh
cargo verset dependency tokio ">=1.40, <2"
```

Only some tables or members:

```sh
cargo verset dependency serde 1.0.200 --kind dev --kind build
cargo verset dependency serde 1.0.200 -p my-cli
```

With a registry:

```sh
cargo verset dependency serde 1.0.200 --registry my-registry
```

### Dry run

```sh
cargo verset package 1.2.3 --dry-run
```

Prints every `old -> new` change per manifest without writing anything.

## Migrating from 0.3

- Both commands now change the whole workspace instead of a single `Cargo.toml`. Use `--package` / `--exclude` to narrow it down.
- The version and the dependency name are positional: `cargo verset package 1.2.3` and `cargo verset dependency serde 1.0.200`. `--ver` and `--name` still work.
- `-v` now means `--verbose` and `-p` now means `--package`. Use `--ver` and `--manifest-path` (or `--path`) instead.
- `dependency` no longer adds a dependency that does not exist, use `cargo add` for that.
- `dependency` keeps the `path` of a path dependency and adds the version next to it.
- When nothing could be changed the command fails with exit code 1 instead of printing a warning.
