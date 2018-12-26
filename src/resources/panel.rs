use serde::Deserialize;

use crate::constants::{Area, Mode};

#[derive(Deserialize)]
pub struct Status {
    forms: Forms,
}

impl Status {
    pub fn inner(self) -> ((Area, Mode), (Area, Mode)) {
        (
            (Area::Area1, self.forms.pcondform1.mode),
            (Area::Area2, self.forms.pcondform2.mode),
        )
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
