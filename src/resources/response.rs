use serde::Deserialize;

use crate::constants::Status;
use crate::errors::{Error, Result};
use crate::resources::ApiResponse;

#[derive(Deserialize, Debug)]
pub struct Response {
    result: Status,
    message: String,
}

impl ApiResponse for Response {
    type Type = String;

    fn into_result(self) -> Result<Self::Type> {
        if self.result == Status::Error {
            return Err(Error::Panel(self.message));
        }

        Ok(self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response() {
        let json = serde_json::json!({ "result": 1, "message": "token123" });
        let resp: Response = serde_json::from_value(json).unwrap();
        assert_eq!(resp.into_result().unwrap(), "token123");
    }

    #[test]
    fn error_response() {
        let json = serde_json::json!({ "result": 0, "message": "something failed" });
        let resp: Response = serde_json::from_value(json).unwrap();
        let err = resp.into_result().unwrap_err();
        assert!(matches!(err, Error::Panel(msg) if msg == "something failed"));
    }
}
