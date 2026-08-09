use serde::{Deserialize, Serialize};

use std::ops::Index;

use crate::constants::{Area, Mode};
use crate::errors::Result;
use crate::resources::ApiResponse;

/// Represents the status of Area 1 and 2.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modes {
    /// Mode of Area 1
    pub area1: Mode,

    /// Mode of Area 2
    pub area2: Mode,
}

/// Look up an area's mode dynamically: `modes[Area::Area1]`.
impl Index<Area> for Modes {
    type Output = Mode;

    fn index(&self, area: Area) -> &Mode {
        match area {
            Area::Area1 => &self.area1,
            Area::Area2 => &self.area2,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Condition {
    forms: Forms,
}

impl ApiResponse for Condition {
    type Output = Modes;

    fn into_result(self) -> Result<Self::Output> {
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
        assert_eq!(
            modes,
            Modes {
                area1: Mode::Disarmed,
                area2: Mode::Armed,
            }
        );
    }

    #[test]
    fn index_by_area() {
        let modes = Modes {
            area1: Mode::Disarmed,
            area2: Mode::Armed,
        };
        assert_eq!(modes[Area::Area1], Mode::Disarmed);
        assert_eq!(modes[Area::Area2], Mode::Armed);
    }
}
