use serde::Deserialize;

use crate::constants::Status;
use crate::errors::{Error, Result as R};
use crate::resources::ApiResponse;

#[derive(Deserialize, Debug)]
pub struct Result {
    result: Status,
    message: String,
}

impl ApiResponse for Result {
    type Type = String;

    fn ok(self) -> R<Self::Type> {
        if let Status::Error = self.result {
            return Err(Error::Panel(self.message));
        }

        Ok(self.message)
    }
}
