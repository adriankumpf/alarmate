// #![deny(warnings)]

mod client;
mod constants;
mod resources;

#[macro_use]
mod utils;

pub use self::client::Client;
pub use self::constants::{Area, Mode, Status};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
