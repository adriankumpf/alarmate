//! This crate provides Rust bindings to the Lupusec HTTP API.
//!
//! ## Getting Started
//!
//! To get started, we need to create a client:
//!
//! ```rust
//!   let ip_address = "192.168.178.10".parse().unwrap();
//!   let client = alarmate::Client::new("admin", "changeme", ip_address).unwrap();
//! ```

#![deny(missing_docs)]

#[macro_use]
mod utils;
mod client;
mod constants;
mod errors;
mod resources;

pub use client::Client;
pub use constants::{Area, DeviceKind, Mode};
pub use errors::{Error, Result};
pub use resources::{devices::Device, panel::Modes};
