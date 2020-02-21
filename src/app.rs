#![allow(unused_imports)]

use anyhow::{bail, Context, Result};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use yew::{
    events::InputData,
    format::Json as YewJson,
    html,
    html::Renderable,
    services::storage::{Area, StorageService},
    services::ConsoleService,
    Component, ComponentLink, Html, ShouldRender,
};

lazy_static! {
    static ref LOCAL_STORAGE_KEY: String =
        { format!("totorigolo.{}.state", env!("CARGO_PKG_NAME")) };
}

const JSON_INPUT: &str = include_str!("input_data.json");
const INPUT_TEMPLATE: &str = include_str!("input_template.hbs");

pub struct Model {
    link: ComponentLink<Self>,
    console: ConsoleService,
    template_engine: HandlebarsEngine,
    storage: StorageService,
    state: State,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum State {
    Init,
    LoadFailed(String),
    Loaded {
        template: String,
        inputs: Vec<Input>,
        #[serde(default)]
        inputs_data: InputsData,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Input {
    key: Path,
    name: String,
    description: Option<String>,
}

type Path = std::rc::Rc<Vec<String>>;

/// Represents the data entered in the inputs on the page.
///
/// Backed by a JSON object.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct InputsData(JsonValue);

impl Default for InputsData {
    fn default() -> Self {
        Self(JsonValue::Object(Default::default()))
    }
}

/// TODO
///
/// # Panic
/// Must be called on a `JsonValue::Object` variant.
fn insert_at_json<T>(json: &mut JsonValue, path: &[T], value: JsonValue)
where
    T: AsRef<str>,
{
    if let JsonValue::Object(obj) = json {
        if let [tail] = path {
            obj.insert(tail.as_ref().to_string(), value);
        } else {
            let nested_obj = obj
                .entry(path[0].as_ref())
                .or_insert(JsonValue::Object(Default::default()));
            insert_at_json(nested_obj, &path[1..], value)
        }
    } else {
        unreachable!("Compile-time logic error: json argument of insert_at_json must be an object.")
    }
}

/// TODO
///
/// # Panic
/// Must be called on a `JsonValue::Object` variant.
fn get_at_json<'a, T: AsRef<str>>(json: &'a JsonValue, path: &[T]) -> Option<&'a JsonValue> {
    if let JsonValue::Object(obj) = json {
        if let [tail] = path {
            obj.get(tail.as_ref())
        } else {
            obj.get(path[0].as_ref())
                .and_then(|nested_obj| get_at_json(nested_obj, &path[1..]))
        }
    } else {
        unreachable!("Compile-time logic error: json argument of insert_at_json must be an object.")
    }
}

impl InputsData {
    fn insert_at<T: AsRef<str> + std::fmt::Debug>(&mut self, path: &[T], value: JsonValue) {
        insert_at_json(&mut self.0, path, value)
    }

    fn get_at<'a, T: AsRef<str>>(&'a self, path: &[T]) -> Option<&'a JsonValue> {
        get_at_json(&self.0, path)
    }
}

#[derive(Debug)]
pub enum Msg {
    Init,
    FetchedJsonData(String),
    LoadFromLocalStorage,
    SaveToLocalStorage,
    EditedInput(Path, String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::LoadFromLocalStorage);

        Model {
            link,
            console: ConsoleService::new(),
            template_engine: HandlebarsEngine::new_uninit(),
            state: State::Init,
            storage: StorageService::new(Area::Local),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.console.log(&format!("Received: {:?}", msg));
        match msg {
            Msg::Init => {
                let json_str =
                    JSON_INPUT.replace("%TEMPLATE%", &INPUT_TEMPLATE.replace("\n", "\\n"));
                self.link.send_message(Msg::FetchedJsonData(json_str));
                false
            }
            Msg::FetchedJsonData(json_str) => {
                let mut json_data: JsonValue =
                    serde_json::from_str(&json_str).expect("Invalid JSON.");
                let template = json_data["template"]
                    .as_str()
                    .expect("JSON input must have a template.")
                    .to_string();

                if let Err(e) = self.template_engine.set_template(&template) {
                    self.state = State::LoadFailed(format!("Received invalid template: {:#?}", e));
                } else {
                    self.state = State::Loaded {
                        template,
                        inputs: serde_json::from_value(json_data["inputs"].take())
                            .expect("Failed to deserialize inputs"),
                        inputs_data: InputsData::default(),
                    };
                    self.link.send_message(Msg::SaveToLocalStorage);
                }
                true
            }
            Msg::LoadFromLocalStorage => {
                if let YewJson(Ok(restored_state)) =
                    self.storage.restore(LOCAL_STORAGE_KEY.as_ref())
                {
                    self.state = restored_state;
                    if let State::Loaded { template, .. } = &self.state {
                        if let Err(e) = self.template_engine.set_template(template) {
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
            Msg::EditedInput(key, value) => match &mut self.state {
                State::Loaded { inputs_data, .. } => {
                    inputs_data.insert_at(key.as_slice(), JsonValue::String(value));
                    self.link.send_message(Msg::SaveToLocalStorage);
                    true
                }
                _ => {
                    self.console.warn(&format!(
                        "Shouldn't have received a Msg::EditedInput message in {:?} state.",
                        self.state
                    ));
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
                inputs,
                inputs_data,
                ..
            } => {
                html! {
                    <div class="columns">
                        <div class="column">
                            { render_inputs(inputs, inputs_data, &self.link) }
                        </div>
                        <div class="column">
                            { render_code_column(inputs_data, &self.template_engine) }
                        </div>
                    </div>
                }
            }
        }
    }
}

pub trait TemplateEngine {
    fn render<T: Serialize>(&self, data: &T) -> Result<String>;
}

pub struct HandlebarsEngine {
    inner: Handlebars<'static>,
}

impl HandlebarsEngine {
    pub fn new_uninit() -> Self {
        Self {
            inner: Handlebars::default(),
        }
    }

    #[allow(unused)]
    pub fn with_template<S: AsRef<str>>(template: S) -> Self {
        let mut s = Self::new_uninit();
        s.set_template(template);
        s
    }

    pub fn set_template<S: AsRef<str>>(&mut self, template: S) -> Result<()> {
        self.inner
            .register_template_string("t", template)
            .context("Handlebars engine failed to compile the template")
    }

    #[allow(unused)]
    fn is_initialized(&self) -> bool {
        self.inner.has_template("t")
    }
}

impl TemplateEngine for HandlebarsEngine {
    fn render<T: Serialize>(&self, data: &T) -> Result<String> {
        self.inner
            .render("t", &data)
            .context("Handlebars template engine failed to render data")
    }
}

fn render_input(input: &Input, inputs_data: &InputsData, link: &ComponentLink<Model>) -> Html {
    let key_for_callback = input.key.clone();
    let on_input = link.callback(move |input_data: InputData| {
        Msg::EditedInput(key_for_callback.clone(), input_data.value)
    });

    let field_help = input
        .description
        .as_ref()
        .map(|txt| html! { <p class="help">{txt}</p> })
        .unwrap_or_default();

    let value = inputs_data
        .get_at(&input.key)
        .map(|val: &_| match val {
            JsonValue::Null => "".to_owned(),
            JsonValue::Bool(true) => "true".to_owned(),
            JsonValue::Bool(false) => "false".to_owned(),
            JsonValue::Number(n) => format!("{}", n),
            JsonValue::String(s) => s.clone(),
            _ => format!("{}", val),
        })
        .unwrap_or_default();

    html! {
        <div class="field">
            <label class="label">{&input.name}</label>
            <div class="control">
                <input class="input" type="text" placeholder={&input.name} oninput=&on_input value={value} />
            </div>
            { field_help }
        </div>
    }
}

fn render_inputs(inputs: &[Input], inputs_data: &InputsData, link: &ComponentLink<Model>) -> Html {
    html! {
        <>
            { for inputs.iter().map(|input| render_input(input, inputs_data, link)) }
        </>
    }
}

fn render_code_column<T: TemplateEngine>(inputs_data: &InputsData, template_engine: &T) -> Html {
    let rendered = template_engine
        .render(inputs_data)
        .unwrap_or_else(|e| format!("Failed to render the data: {:#?}", e));

    html! {
        <>
            <pre>{rendered}</pre>
        </>
    }
}
