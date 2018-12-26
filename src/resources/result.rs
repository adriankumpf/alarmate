use serde::Deserialize;

use crate::constants::Status;
use crate::err;
use crate::Result as MyResult;

#[derive(Deserialize, Debug)]
pub struct Result {
    result: Status,
    message: String,
}

impl Result {
    pub fn ok(self) -> MyResult<String> {
        if let Status::Error = self.result {
            return err!("C: {}", self.message);
        }

        Ok(self.message)
    }
}
