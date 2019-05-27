use std::fmt;
use std::io;

/// A `Result` alias where the `Err` case is `alarmate::Error`
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Possible Errors
#[derive(Debug)]
pub enum Error {
    /// An error reported by the alarm panel
    Panel(String),

    /// A deserilization error
    Deserialize(serde_json::error::Error),

    /// An error converting a header from a string
    InvalidHeader(reqwest::header::InvalidHeaderValue),

    /// A networking error communicating with the alarm panel
    Http(reqwest::Error),

    /// An error constructing a URL
    Url(reqwest::UrlError),

    /// An error for I/O operations
    Io(io::Error),
}

impl Error {
    /// Indicates whether an error represents a session timeout issued by the lupusec panel.
    pub fn is_session_timeout(&self) -> bool {
        match *self {
            Error::Panel(ref err)
                if err == "401 Unauthorized: Zugriff verweigert: Sitzung abgelaufen!" =>
            {
                true
            }
            _ => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(std::error::Error::description(self))?;
        match *self {
            Error::Panel(ref msg) => write!(f, "{}", msg),
            Error::Deserialize(ref msg) => write!(f, "{}", msg),
            Error::InvalidHeader(ref msg) => write!(f, "{}", msg),
            Error::Http(ref err) => write!(f, ": {}", err),
            Error::Url(ref err) => write!(f, ": {}", err),
            Error::Io(ref err) => write!(f, ": {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Panel(_) => "error reported by the alarm panel",
            Error::Deserialize(_) => "error deserializing panel response",
            Error::InvalidHeader(_) => "error converting a header from a string",
            Error::Http(_) => "error communicating with the panel",
            Error::Url(_) => "error constructing a URL",
            Error::Io(_) => "error for I/O operations",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Error::Panel(_) => None,
            Error::Deserialize(ref err) => Some(err),
            Error::InvalidHeader(ref err) => Some(err),
            Error::Http(ref err) => Some(err),
            Error::Url(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::Deserialize(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Error {
        Error::InvalidHeader(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Http(err)
    }
}

impl From<reqwest::UrlError> for Error {
    fn from(err: reqwest::UrlError) -> Error {
        Error::Url(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
