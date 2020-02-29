use crate::{for_all_inputtypes_variants, prelude::*, Path};

mod boolean;
mod group;
mod list;
mod macros;
mod number;
mod text;
pub use boolean::*;
pub use group::*;
pub use list::*;
pub use number::*;
pub use text::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InputTypes {
    Text(TextInput),
    Boolean(BooleanInput),
    Number(NumberInput),
    Group(GroupInput),
    /// List differs from groups in that the number of input can
    /// varry, eg. it can be used to prompt for a list of persons
    /// of unknown size.
    List(ListInput),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputInfo {
    pub key: Path,
    pub name: String,
    pub description: Option<String>,
}

pub trait Input {
    fn key(&self) -> &Path;

    fn name(&self) -> &str;

    fn description(&self) -> Option<&str>;
}

impl Input for InputTypes {
    fn key(&self) -> &Path {
        for_all_inputtypes_variants! { self, i => i.key() }
    }

    fn name(&self) -> &str {
        for_all_inputtypes_variants! { self, i => i.name() }
    }

    fn description(&self) -> Option<&str> {
        for_all_inputtypes_variants! { self, i => i.description() }
    }
}
