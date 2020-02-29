use super::{InputInfo, InputTypes};
use crate::impl_input_for;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ListInput {
    #[serde(flatten)]
    pub info: InputInfo,
    pub inputs: Vec<InputTypes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
}

impl_input_for!(ListInput);
