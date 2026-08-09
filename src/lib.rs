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
//!
//! [`Client`] methods take `&self`, so one client can be shared across tasks
//! behind an [`Arc`](std::sync::Arc).

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

mod client;
mod constants;
mod errors;
mod resources;
mod utils;

pub use client::Client;
pub use constants::{Area, DeviceKind, Mode, State, Status};
pub use errors::{Error, Result};
pub use resources::{devices::Device, panel::Modes};

/// Re-exported so [`Error::UnexpectedResponse`] can be inspected without
/// depending on `reqwest` directly.
pub use reqwest::StatusCode;
