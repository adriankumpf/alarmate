use thiserror::Error;

/// A `Result` alias where the `Err` case is `alarmate::Error`
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Possible Errors
///
/// Non-exhaustive: more variants will be added as more of the panel's API is
/// covered.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error reported by the alarm panel
    #[error("error reported by the alarm panel: {0}")]
    Panel(String),

    /// An authentication error (invalid credentials)
    #[error("unauthorized: invalid credentials")]
    Unauthorized,

    /// A session timeout error
    #[error("the session expired")]
    SessionTimeout,

    /// An unexpected response error
    #[error("received an unexpected response with status {status}: {body}")]
    #[non_exhaustive]
    UnexpectedResponse {
        /// The HTTP status code of the response
        status: reqwest::StatusCode,
        /// The body of the HTTP response
        body: String,
    },

    /// A deserialization error
    #[error("error deserializing panel response: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// An error converting a header from a string
    #[error("error converting a header from a string: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// A networking error communicating with the alarm panel
    #[error("error communicating with the panel: {0}")]
    Http(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(Error::SessionTimeout.to_string(), "the session expired");
        assert_eq!(
            Error::Unauthorized.to_string(),
            "unauthorized: invalid credentials"
        );
        assert!(Error::Panel("oops".into()).to_string().contains("oops"));
    }
}
