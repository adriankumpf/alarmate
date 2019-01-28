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

pub use client::Client;
pub use constants::{Area, DeviceKind, Mode, Status};
pub use errors::{Error, Result};
pub use resources::{devices::Device, panel::Modes};
