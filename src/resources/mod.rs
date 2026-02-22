pub mod devices;
pub mod panel;
pub mod response;

use crate::errors::Result;

/// A trait implemented by raw API response types to convert themselves into
/// the domain type the caller actually needs.
///
/// Each LUPUSEC endpoint returns a JSON wrapper with extra metadata. Implement
/// this trait on the deserialized wrapper so that [`Client`](crate::Client)
/// methods can uniformly extract the inner value.
pub trait ApiResponse {
    /// The domain type produced after validating and unwrapping the response.
    type Type;

    /// Validate the response and extract the inner value.
    fn into_result(self) -> Result<Self::Type>;
}
