use thiserror::Error;

/// A `Result` alias where the `Err` case is `alarmate::Error`
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Possible Errors
#[derive(Error, Debug)]
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

impl Error {
    pub(crate) fn is_session_timeout(&self) -> bool {
        matches!(*self, Error::SessionTimeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_timeout_is_session_timeout() {
        assert!(Error::SessionTimeout.is_session_timeout());
    }

    #[test]
    fn other_errors_are_not_session_timeout() {
        assert!(!Error::Unauthorized.is_session_timeout());
        assert!(!Error::Panel("test".into()).is_session_timeout());
        assert!(
            !Error::Deserialize(serde_json::from_str::<()>("bad").unwrap_err())
                .is_session_timeout()
        );
    }

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
