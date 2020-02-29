use crate::{inputs::InputTypes, prelude::*};

/// A scenario represents the template to be rendered and the format
/// of inputs needed to generate it.
#[derive(Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub template: String,
    pub inputs: Vec<InputTypes>,
}

impl Scenario {}

#[allow(unused)]
pub struct ScenarioAsJson<'a>(&'a Scenario);
