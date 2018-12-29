pub mod devices;
pub mod panel;
pub mod result;

// pub use self::devices;
// pub use self::panel;
// pub use self::result;

use crate::errors;

pub trait ApiResponse {
    type Type: serde::de::DeserializeOwned;

    fn ok(self) -> errors::Result<Self::Type>;
}
