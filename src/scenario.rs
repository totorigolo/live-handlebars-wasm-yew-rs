use crate::{inputs::InputTypes, prelude::*};

/// A scenario represents the template to be rendered and the format
/// of inputs needed to generate it.
#[derive(Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub template: Template,
    pub inputs: Vec<InputTypes>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Template {
    StringTemplate(String),
    StringListTemplate(Vec<String>),
}

impl Scenario {}

#[allow(unused)]
pub struct ScenarioAsJson<'a>(&'a Scenario);
