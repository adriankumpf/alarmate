use serde::de::DeserializeOwned;

pub mod devices;
pub mod panel;
pub mod response;

// pub use self::devices;
// pub use self::panel;
// pub use self::response;

use crate::errors::Result;

pub trait ApiResponse {
    type Type: DeserializeOwned;

    fn ok(self) -> Result<Self::Type>;
}
