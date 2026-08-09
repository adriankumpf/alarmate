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
alarmate = { git = "https://github.com/adriankumpf/alarmate", tag = "v0.5.0" }
```

## Usage

### Binary

```bash
$ alarmate --help

A Rust client for the LUPUSEC XT2 alarm panel API

Usage: alarmate <COMMAND>

Commands:
  devices  List devices
  status   Get current status
  mode     Change mode
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Connection options (`-I`, `-U`, `-P`) are required for all commands and can also
be set via environment variables:

| Flag               | Environment Variable  |
| ------------------ | --------------------- |
| `-I, --ip-address` | `ALARMATE_IP_ADDRESS` |
| `-U, --username`   | `ALARMATE_USERNAME`   |
| `-P, --password`   | `ALARMATE_PASSWORD`   |

Prefer `ALARMATE_PASSWORD` over `-P`: command-line arguments are visible to
every other user on the machine via `ps`.

### Library

```rust
use alarmate::{Area, Client, Mode};

#[tokio::main]
async fn main() -> alarmate::Result {
    let client = Client::new("admin", "changeme", "192.168.178.10".parse().unwrap())?;
    client.change_mode(Area::Area1, Mode::Disarmed).await?;
    Ok(())
}
```

All methods take `&self`, so a single client can be shared across tasks behind
an `Arc`.

> [!WARNING]
> LUPUSEC panels ship with self-signed certificates, so the client does not
> validate them. Only use it on a network you trust.

## Documentation

```bash
cargo doc --open
```

## Project status

This library only supports the XT2 alarm panel. Other LUPUSEC alarm panels
probably won't work due to differing APIs.

Currently there is only a limited feature set available. Please open a PR or an
issue if you feel there is something missing!

## License

[MIT](https://choosealicense.com/licenses/mit/)
