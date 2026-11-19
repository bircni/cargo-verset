# Changelog - [cargo-verset](https://github.com/bircni/cargo-verset)

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

## [0.4.0](https://github.com/bircni/cargo-verset/compare/0.3.2..0.4.0) - 2026-11-19

### Documentation

- describe workspace-wide usage and migration from 0.3 - ([e5c6fc1](https://github.com/bircni/cargo-verset/commit/e5c6fc19d6114303d24c499558a555797b33210e)) - bircni

### Features

-  [**breaking**]set versions across the whole workspace - ([9b71754](https://github.com/bircni/cargo-verset/commit/9b71754c97ed70cb96ced6c5d06726805ee9d377)) - bircni

### Miscellaneous Chores

- **(deps)** update dependencies to latest (#9) - ([8bfa8f5](https://github.com/bircni/cargo-verset/commit/8bfa8f5ca1a778220bc25c41af8f8d74bbbb7a75)) - bircni
- **(deps)** update dependencies to latest - ([a39a9d7](https://github.com/bircni/cargo-verset/commit/a39a9d73a07692f632d1fee744ef4d6a7229bdc7)) - bircni
- Update dependencies - ([ec64e5f](https://github.com/bircni/cargo-verset/commit/ec64e5fae978104a4ae82cbe42edc9bf91d77246)) - Nicolas

### Build

- use --ver in the release scripts - ([07f4526](https://github.com/bircni/cargo-verset/commit/07f4526b87fd9bf08318eb6aa6250c51a4ced9cc)) - bircni

## [0.3.2](https://github.com/bircni/cargo-verset/compare/0.3.1..0.3.2) - 2025-12-07

### Bug Fixes

- update dependencies and improve documentation with examples - ([ad400bf](https://github.com/bircni/cargo-verset/commit/ad400bf581ef174ec87d73efef6962e0a203da1a)) - Nicolas

## [0.3.1](https://github.com/bircni/cargo-verset/compare/0.3.0..0.3.1) - 2025-10-12

### Bug Fixes

- update dependencies and exclude additional directories from package - ([c0681df](https://github.com/bircni/cargo-verset/commit/c0681dfde785fa33d318bfb94e1495504377ae05)) - Nicolas

## [0.3.0](https://github.com/bircni/cargo-verset/compare/0.2.0..0.3.0) - 2025-07-23

### Features

- enhance dependency version handling by setting version explicitly when path is present - ([534f138](https://github.com/bircni/cargo-verset/commit/534f138298560e7f1c50304dffcd2559b79dcf91)) - Nicolas Bircks
- add an option to set a registry - ([36ced63](https://github.com/bircni/cargo-verset/commit/36ced63f52d232293c271e1bd322f4508e8cd60f)) - Nicolas Bircks

### Build

- remove old script - ([dca471b](https://github.com/bircni/cargo-verset/commit/dca471b0e9d8ad9960b29fb469f0ce75e6387af3)) - Nicolas
- update toolchain to stable and bump cargo-shear version to 1.4.0 - ([f353933](https://github.com/bircni/cargo-verset/commit/f3539335a5822410b900d52f1e4c23e7dab40454)) - Nicolas

## [0.2.0](https://github.com/bircni/cargo-verset/compare/0.1.6..0.2.0) - 2025-06-25

### Features

-  [**breaking**]implement CLI for managing package and dependency versions in Cargo.toml - ([a7bde0f](https://github.com/bircni/cargo-verset/commit/a7bde0fc79618a601cd4c96a61d0611d8b877a99)) - Nicolas

### Tests

- also test with insta - ([b485d85](https://github.com/bircni/cargo-verset/commit/b485d853e2dcfd3a4942d40068d52965691b766d)) - Nicolas

### Build

- update to rust 1.87 (#3) - ([e9d756f](https://github.com/bircni/cargo-verset/commit/e9d756f3ce9395bd0f9f921df99459761a770e75)) - Nicolas
- switch to cargo shear - ([743691c](https://github.com/bircni/cargo-verset/commit/743691cc0252183d2431abe8194c501849617263)) - Nicolas

## [0.1.6](https://github.com/bircni/cargo-verset/compare/0.1.5..0.1.6) - 2025-05-10

### Miscellaneous Chores

- cleanup some things - ([9ddfd94](https://github.com/bircni/cargo-verset/commit/9ddfd9451928ad6852d37fcd7a19981aeb08f4d7)) - Nicolas

## [0.1.5](https://github.com/bircni/cargo-verset/compare/0.1.4..0.1.5) - 2025-04-19

### Documentation

- add Changelog generation on release - ([5881f49](https://github.com/bircni/cargo-verset/commit/5881f494ed6040811e4e72201253a266a85dc1f3)) - Nicolas
- add more commit parsing options - ([1c90f03](https://github.com/bircni/cargo-verset/commit/1c90f0381dc1f660135d9bd262fde5443973ad04)) - Nicolas

### Miscellaneous Chores

- **(cleanup)** remove unnecessary tests - ([a3fba7e](https://github.com/bircni/cargo-verset/commit/a3fba7ebca8a3b34e25f38d13bac74dffaef4978)) - Nicolas

### Style

- add build skript for exact version on build time - ([2fa43bb](https://github.com/bircni/cargo-verset/commit/2fa43bb051569cc811e475f756fb2ed1181b5baa)) - Nicolas

### Tests

- enhance lints for clippy - ([6be97df](https://github.com/bircni/cargo-verset/commit/6be97df99734cc3efeee8229f0f3c89ba67099db)) - Nicolas

## [0.1.0] - 2025-01-01
