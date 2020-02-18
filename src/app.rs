use handlebars::Handlebars;
use std::collections::HashMap;
use yew::{
    events::InputData, html, services::ConsoleService, Component, ComponentLink, Html, ShouldRender,
};

const TEMPLATE: &str = "Hello {{first_name}} {{last_name}}";

pub struct App {
    link: ComponentLink<Self>,
    console: ConsoleService,
    handlebars: Handlebars<'static>,
    model: Model,
}

type Model = HashMap<String, String>;

pub enum Msg {
    Edited(String, String),
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

        App {
            link,
            console: ConsoleService::new(),
            handlebars,
            model: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Edited(key, value) => {
                self.console
                    .log(&format!("Received: Msg::Edited({}, {})", key, value));
                self.model.insert(key, value);
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

fn render_inputs(app: &App) -> Html {
    let on_edit = |key| {
        app.link.callback(move |input_data: InputData| {
            Msg::Edited(String::from(key), input_data.value)
        })
    };

    let inputs = [("first_name", "First Name"), ("last_name", "Last Name")]
        .iter()
        .map(move |(key, name)| {
            html! {
                <input class="input" type="text" placeholder={name} oninput=on_edit(*key) />
            }
        })
        .collect::<Vec<Html>>();

    html! {
        <>
            { for inputs }
        </>
    }
}

fn render_code_column(app: &App) -> Html {
    let rendered = app.handlebars
        .render("template", &app.model)
        .expect("Failed to render template");

    html! {
        <>
            <pre>{rendered}</pre>
        </>
    }
}
