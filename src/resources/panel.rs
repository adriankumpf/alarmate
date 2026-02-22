use serde::{Deserialize, Serialize};

use crate::constants::Mode;
use crate::errors::Result;
use crate::resources::ApiResponse;

/// Represents the status of Area 1 and 2.
#[derive(Serialize, Deserialize, Debug)]
pub struct Modes {
    /// Mode of Area 1
    pub area1: Mode,

    /// Mode of Area 2
    pub area2: Mode,
}

#[derive(Deserialize)]
pub struct Condition {
    forms: Forms,
}

impl ApiResponse for Condition {
    type Type = Modes;

    fn ok(self) -> Result<Self::Type> {
        Ok(Modes {
            area1: self.forms.pcondform1.mode,
            area2: self.forms.pcondform2.mode,
        })
    }
}

#[derive(Deserialize)]
struct Forms {
    pcondform1: PCondForm,
    pcondform2: PCondForm,
}

#[derive(Deserialize)]
struct PCondForm {
    mode: Mode,
}
