//! This crate provides Rust bindings to the Lupusec HTTP API.
//!
//! ## Getting Started
//!
//! To get started, we need to create a client:
//!
//! ```rust
//!   let client = alarmate::Client::new("admin", "changeme", "192.168.178.10");
//! ```

#![deny(missing_docs)]

mod client;
mod constants;
mod errors;
mod resources;

#[macro_use]
mod utils;

pub use self::client::Client;
pub use self::constants::{Area, DeviceKind, Mode, Status};
pub use self::errors::{Error, Result};
pub use self::resources::devices::Device;
