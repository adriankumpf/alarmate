use thiserror::Error;

/// A `Result` alias where the `Err` case is `alarmate::Error`
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Possible Errors
#[derive(Error, Debug)]
pub enum Error {
    /// An error reported by the alarm panel
    #[error("error reported by the alarm panel: {0}")]
    Panel(String),

    /// A deserilization error
    #[error("error deserializing panel response: {0:?}")]
    Deserialize(String),

    /// An error converting a header from a string
    #[error("error converting a header from a string: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// A networking error communicating with the alarm panel
    #[error("error communicating with the panel: {0}")]
    Http(#[from] reqwest::Error),

    /// An error for I/O operations
    #[error("error for I/O operations: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Indicates whether an error represents a session timeout issued by the lupusec panel.
    pub fn is_session_timeout(&self) -> bool {
        matches!(
            *self,
            Error::Panel(ref err) if err == "401 Unauthorized: Zugriff verweigert: Sitzung abgelaufen!"
        )
    }
}
