use crate::{
    prelude::*,
    scenario::Scenario,
    template_engine::{HandlebarsEngine, TemplateEngine},
    InputsData, Path,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use yew::{
    format::Json as YewJson,
    services::storage::{Area, StorageService},
    Component, ComponentLink, Html, ShouldRender,
};

use crate::inputs::*;

lazy_static! {
    static ref LOCAL_STORAGE_KEY: String =
        { format!("totorigolo.{}.state", env!("CARGO_PKG_NAME")) };
}

const JSON_INPUT: &str = include_str!("input_data.json");
const INPUT_TEMPLATE: &str = include_str!("input_template.hbs");

pub struct Model {
    link: ComponentLink<Self>,
    template_engine: HandlebarsEngine,
    storage: StorageService,
    state: State,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    Init,
    LoadFailed(String),
    Loaded {
        scenario: Scenario,
        #[serde(default)]
        inputs_data: InputsData,
    },
}

#[derive(Debug)]
pub enum Msg {
    Init,
    FetchedJsonData(String),
    LoadFromLocalStorage,
    SaveToLocalStorage,
    EditedInput(Path, JsonValue),
    ListInputSizeChanged(Path, usize),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::LoadFromLocalStorage);

        Model {
            link,
            template_engine: HandlebarsEngine::new_uninit(),
            state: State::Init,
            storage: StorageService::new(Area::Local),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        trace!("Received: {:?}", msg);
        match msg {
            Msg::Init => {
                let json_str =
                    JSON_INPUT.replace("%TEMPLATE%", &INPUT_TEMPLATE.replace("\n", "\\n"));
                self.link.send_message(Msg::FetchedJsonData(json_str));
                false
            }
            Msg::FetchedJsonData(json_str) => match self.load_from_json(&json_str) {
                Ok(should_render) => should_render,
                Err(e) => {
                    self.state = State::LoadFailed(format!("Received invalid data: {:#?}", e));
                    true
                }
            },
            Msg::LoadFromLocalStorage => {
                if let YewJson(Ok(restored_state)) =
                    self.storage.restore(LOCAL_STORAGE_KEY.as_ref())
                {
                    self.state = restored_state;
                    if let State::Loaded { scenario, .. } = &self.state {
                        if let Err(e) = self.template_engine.set_template(&scenario.template) {
                            self.storage.remove(LOCAL_STORAGE_KEY.as_ref());
                            self.state = State::LoadFailed(format!(
                                "Invalid template fetched from local storage: {:#?}",
                                e
                            ));
                        }
                    }
                    true
                } else {
                    // If we're here, local storage is either absent or invalid
                    self.storage.remove(LOCAL_STORAGE_KEY.as_ref());
                    self.link.send_message(Msg::Init);
                    false
                }
            }
            Msg::SaveToLocalStorage => {
                self.storage
                    .store(LOCAL_STORAGE_KEY.as_ref(), YewJson(&self.state));
                false
            }
            Msg::EditedInput(path, value) => match &mut self.state {
                State::Loaded { inputs_data, .. } => {
                    match inputs_data.insert_at(&path, value) {
                        Ok(()) => self.link.send_message(Msg::SaveToLocalStorage),
                        Err(e) => {
                            // TODO: Show the error
                            error!("Failed to save value of '{}': {:?}", path, e);
                        }
                    }
                    true
                }
                _ => {
                    warn!(
                        "Shouldn't have received a Msg::EditedInput message in state: {:?}.",
                        self.state
                    );
                    false
                }
            },
            Msg::ListInputSizeChanged(path, new_size) => match &mut self.state {
                State::Loaded { inputs_data, .. } => {
                    if let Err(e) = inputs_data.resize_array_at(&path, new_size) {
                        warn!("Failed to access array at '{}': {:?}", path, e);
                    }

                    self.link.send_message(Msg::SaveToLocalStorage);
                    true
                }
                _ => {
                    warn!(
                        "Shouldn't have received a Msg::ListInputSizeChanged message in state: \
                         {:?}.",
                        self.state
                    );
                    false
                }
            },
        }
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Init => {
                html! {
                    <p>{ "Loading..." }</p>
                }
            }
            State::LoadFailed(error_msg) => {
                html! {
                    <p>{ format!("Load failed: {}", error_msg) }</p>
                }
            }
            State::Loaded {
                scenario,
                inputs_data,
                ..
            } => {
                let on_reload = self.link.callback(|_| Msg::Init);
                html! {
                    <>
                        <button class="button" onclick=on_reload>{ "Reload" }</button>
                        <div class="columns">
                            <div class="column">
                                { render_inputs(&scenario.inputs, inputs_data, &self.link) }
                            </div>
                            <div class="column">
                                { render_code_column(inputs_data, &self.template_engine) }
                            </div>
                        </div>
                    </>
                }
            }
        }
    }
}

impl Model {
    fn load_from_json(&mut self, json_str: &str) -> Result<ShouldRender> {
        let mut json_data: JsonValue = serde_json::from_str(&json_str).context("Invalid JSON.")?;
        let template = json_data["template"]
            .as_str()
            .context("JSON input must have a template.")?
            .to_string();

        let inputs = serde_json::from_value(json_data["inputs"].take())
            .context("Failed to deserialize inputs")?;

        if let Err(e) = self.template_engine.set_template(&template) {
            self.state = State::LoadFailed(format!("Received invalid template: {:#?}", e));
        } else {
            self.state = State::Loaded {
                scenario: Scenario { template, inputs },
                inputs_data: InputsData::default(),
            };
            self.link.send_message(Msg::SaveToLocalStorage);
        }
        Ok(true)
    }
}

fn render_inputs(
    inputs: &[InputTypes],
    inputs_data: &InputsData,
    link: &ComponentLink<Model>,
) -> Html {
    use crate::views::RenderableInput;

    html! {
        <>
            { for inputs.iter().map(|input| input.render(&Path::default(), inputs_data, link)) }
            <pre>{ format!("{:#?}", inputs) }</pre>
        </>
    }
}

fn render_code_column<T: TemplateEngine>(inputs_data: &InputsData, template_engine: &T) -> Html {
    let rendered = template_engine
        .render(inputs_data)
        .unwrap_or_else(|e| format!("Failed to render the data: {:#?}", e));

    html! {
        <>
            <pre>{ format!("{:#}", inputs_data) }</pre>
            <pre>{rendered}</pre>
        </>
    }
}
