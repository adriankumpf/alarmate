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
pub(crate) struct Condition {
    forms: Forms,
}

impl ApiResponse for Condition {
    type Type = Modes;

    fn into_result(self) -> Result<Self::Type> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_condition() {
        let json = serde_json::json!({
            "forms": {
                "pcondform1": { "mode": 0 },
                "pcondform2": { "mode": 1 }
            }
        });
        let condition: Condition = serde_json::from_value(json).unwrap();
        let modes = condition.into_result().unwrap();
        assert_eq!(modes.area1, Mode::Disarmed);
        assert_eq!(modes.area2, Mode::Armed);
    }
}
