use super::InputInfo;
use crate::impl_input_for;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TextInput {
    #[serde(flatten)]
    pub info: InputInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_regex: Option<String>, // TODO: use a regex
}

impl_input_for!(TextInput);
