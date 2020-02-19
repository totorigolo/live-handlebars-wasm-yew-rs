use anyhow::{bail, Context, Result};
use handlebars::Handlebars;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use yew::{
    events::InputData, html, html::Renderable, services::ConsoleService, Component, ComponentLink, Html, ShouldRender,
};

const TEMPLATE: &str = r#"Test template:

Variables in the current context: {{date}}-{{time}}
Dot-separated variables: {{author.first_name}} {{author.last_name}}
Disable escaping: {{{unescaped}}}

Array access: {{nodes.[2].[#name]}}

{{#with story}}
  <div class="intro">{{{intro}}}</div>
  <div class="body">{{{body}}}</div>
{{/with}}

Iterate nodes:
{{#each node}}
 - id={{id}} name={{name}}
{{/each}}

{{#if is_active}}
Variable "is_active" is set.
{{else}}
Variable "is_active" is unset.
{{/if}}

{{#unless license}}
No license set.
{{/unless}}
"#;

pub struct App {
    link: ComponentLink<Self>,
    console: ConsoleService,
    handlebars: Handlebars<'static>,
    inputs: Vec<Input>,
    model: Model,
}

#[derive(Serialize, Debug, PartialEq)]
struct Model(JsonValue);

impl Default for Model {
    fn default() -> Self {
        Self(JsonValue::Object(Default::default()))
    }
}

/// TODO
///
/// # Panic
/// Must be called on a `JsonValue::Object` variant.
fn insert_in_json<T: AsRef<str> + std::fmt::Debug>(
    json: &mut JsonValue,
    path: &[T],
    value: JsonValue,
) {
    if let JsonValue::Object(obj) = json {
        if let [tail] = path {
            obj.insert(tail.as_ref().to_string(), value);
        } else {
            let nested_obj = obj
                .entry(path[0].as_ref())
                .or_insert(JsonValue::Object(Default::default()));
            insert_in_json(nested_obj, &path[1..], value)
        }
    } else {
        unreachable!("Compile-time logic error: json argument of insert_in_json must be an object.")
    }
}

impl Model {
    fn insert_in<T: AsRef<str> + std::fmt::Debug>(&mut self, path: &[T], value: JsonValue) {
        insert_in_json(&mut self.0, path, value)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct Input {
    key: Path,
    name: String,
    description: Option<String>,
}

type Path = std::rc::Rc<Vec<String>>;

pub enum Msg {
    Edited(Path, String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // TODO: Move to post-init as async
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("template", TEMPLATE)
            .expect("Failed to compile template.");

        let mut template_data = serde_json::json!({
            "inputs": [
                {"key": ["date"],
                 "name": "Date",
                 "description": "A date, like 29/02/2029.",
                },
                {"key": ["time"],
                 "name": "Time",
                 "description": "A time, like 13:37:00. The seconds are optionals.",
                },
                {"key": ["author", "first_name"],
                 "name": "First Name",
                 "description": "The author's first name",
                },
                {"key": ["author", "last_name"],
                 "name": "Last Name",
                 "description": "The author's last name",
                },
                {"key": ["is_active"],
                 "name": "Is active?",
                 "description": "A boolean value.",
                },
                {"key": ["license"],
                 "name": "The license",
                 "description": "Something for your lawer.",
                },
            ]
        });

        App {
            link,
            console: ConsoleService::new(),
            handlebars,
            model: Model::default(),
            inputs: serde_json::from_value(template_data["inputs"].take()).expect("Failed to deserialize inputs"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Edited(key, value) => {
                self.console
                    .log(&format!("Received: Msg::Edited({:?}, {})", key, value));
                self.model
                    .insert_in(key.as_slice(), JsonValue::String(value));

                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns">
                <div class="column">
                    { render_inputs(&self) }
                </div>
                <div class="column">
                    { render_code_column(&self) }
                </div>
            </div>
        }
    }
}

fn render_input(input: &Input, link: &ComponentLink<App>) -> Html {
    let key_for_callback = input.key.clone();
    let on_input = link
            .callback(move |input_data: InputData| Msg::Edited(key_for_callback.clone(), input_data.value));

    let field_help = input.description.as_ref().map(|txt| html! { <p class="help">{txt}</p> }).unwrap_or_default();

    html! {
        <div class="field">
            <label class="label">{&input.name}</label>
            <div class="control">
                <input class="input" type="text" placeholder={&input.name} oninput=&on_input />
            </div>
            { field_help }
        </div>
    }
}

fn render_inputs(app: &App) -> Html {
    html! {
        <>
            { for app.inputs.iter().map(|input| render_input(input, &app.link)) }
        </>
    }
}

fn render_code_column(app: &App) -> Html {
    let rendered = app
        .handlebars
        .render("template", &app.model)
        .expect("Failed to render template");

    html! {
        <>
            <pre>{rendered}</pre>
        </>
    }
}
