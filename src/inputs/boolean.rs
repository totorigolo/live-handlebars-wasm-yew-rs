use super::InputInfo;
use crate::impl_input_for;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BooleanInput {
    #[serde(flatten)]
    pub info: InputInfo,
}

impl_input_for!(BooleanInput);
