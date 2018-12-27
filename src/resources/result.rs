use serde::Deserialize;

use crate::constants::Status;
use crate::errors;

#[derive(Deserialize, Debug)]
pub struct Result {
    result: Status,
    message: String,
}

impl Result {
    pub fn ok(self) -> errors::Result<String> {
        if let Status::Error = self.result {
            return Err(errors::Error::Panel(self.message));
        }

        Ok(self.message)
    }
}
