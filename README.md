# Rocket Launch

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![GitHub release](https://img.shields.io/github/release/tim-sueberkrueb/rocket-launch.svg)](https://github.com/tim-sueberkrueb/rocket-launch/releases)
[![GitHub issues](https://img.shields.io/github/issues/tim-sueberkrueb/rocket-launch.svg)](https://github.com/tim-sueberkrueb/rocket-launch/issues)
[![Maintained](https://img.shields.io/maintenance/yes/2018.svg)](https://github.com/tim-sueberkrueb/rocket-launch/commits/develop)

Watches a Cargo project for changes and automatically relaunches `cargo run`.

`rocket-launch` will only restart `cargo run` when the compilation succeeds,
thus keeping the application running. This is especially useful for writing
server applications in Rust.

It also only triggers a rebuild when a change in a file with a whitelisted extension happens.

This tool is also pretty minimalistic (`< 200 lines`).
Please use use [cargo watch](https://github.com/passcod/cargo-watch) for more options.

## Dependencies

[Rust](https://www.rust-lang.org) and the following crates and their dependencies are required:

 * [clap](https://github.com/kbknapp/clap-rs)
 * [notify](https://github.com/passcod/notify)

## Installation

From the root of the repository, run:

```bash
cargo install
```

## Usage

In your Cargo project directory, run:

```bash
rocket-launch
```

See `rocket-launch --help` for more options.

## Licensing

Licensed unter the terms of the MIT license.
