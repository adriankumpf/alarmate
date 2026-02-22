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
