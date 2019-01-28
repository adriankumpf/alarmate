# Alarmate

A Rust client for interacting with the LUPUSEC XT2 API.

## Installation

### Binary

```bash
cargo build --release --features="build-binary"
```

### Library

```toml
[dependencies]
alarmate = { git = "https://github.com/adriankumpf/alarmate", tag = "v0.2.0" }
```

## Usage

### Binary

```bash
$ alarmate --help

  USAGE:
      alarmate <SUBCOMMAND>

  FLAGS:
      -h, --help       Prints help information
      -V, --version    Prints version information

  SUBCOMMANDS:
      devices    List devices
      help       Prints this message or the help of the given subcommand(s)
      mode       Change mode
      status     Get current status
```

### Library

```rust
use alarmate::{Area, Client, Mode};

let mut client = Client::new("admin", "changeme", "10.0.0.10".parse()?);
client.change_mode(Area::Area1, Mode::Disarmed)?;
```

## Documentation

```bash
cargo doc --open
```

## Project status

This library only supports the XT2 alarm panel. Other LUPUSUC alarm panels
probably won't work due to differing APIs.

Currently there is only a limited feature set available. Please open a PR or an
issue if you feel there is something missing!

## License

[MIT](https://choosealicense.com/licenses/mit/)
