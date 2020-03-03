#![recursion_limit = "512"]

mod agents;
pub mod app;
mod components;
mod inputs;
mod json_path;
mod prelude;
mod scenario;
mod template_engine;
mod views;

pub use json_path::{InputsData, Path};
