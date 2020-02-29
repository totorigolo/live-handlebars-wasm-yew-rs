use super::InputInfo;
use crate::impl_input_for;
use serde::{Deserialize, Serialize};

pub type JsonNumber = serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
pub struct NumberInput {
    #[serde(flatten)]
    pub info: InputInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<JsonNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<JsonNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<JsonNumber>,
}

impl_input_for!(NumberInput);
