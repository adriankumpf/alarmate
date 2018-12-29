use serde::Deserialize;

use crate::constants::{Area, Mode};
use crate::errors::Result;
use crate::resources::ApiResponse;

#[derive(Deserialize)]
pub struct Status {
    forms: Forms,
}

impl ApiResponse for Status {
    type Type = ((Area, Mode), (Area, Mode));

    fn ok(self) -> Result<Self::Type> {
        Ok((
            (Area::Area1, self.forms.pcondform1.mode),
            (Area::Area2, self.forms.pcondform2.mode),
        ))
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
