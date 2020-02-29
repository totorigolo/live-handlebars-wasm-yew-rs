use super::{InputInfo, InputTypes};
use crate::impl_input_for;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupInput {
    #[serde(flatten)]
    pub info: InputInfo,
    pub inputs: Vec<InputTypes>,
    #[serde(default)]
    pub show_disable_toggle: bool,
}

impl_input_for!(GroupInput);
