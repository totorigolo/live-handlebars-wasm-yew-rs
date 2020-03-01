use crate::{app, for_all_inputtypes_variants, inputs::*, prelude::*, InputsData, Path};

type AppComponentLink = yew::ComponentLink<app::Model>;

pub trait RenderableInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html;
}

impl RenderableInput for InputTypes {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        for_all_inputtypes_variants! { self, i => i.render(key_base, inputs_data, link) }
    }
}

impl RenderableInput for TextInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();

        let key_inner = key.clone();
        let on_input = link.callback(move |input_data: InputData| {
            app::Msg::EditedInput(key_inner.clone(), JsonValue::String(input_data.value))
        });

        let value = if let Some(value) = inputs_data.get_at(&key) {
            match value {
                JsonValue::Null => "".to_owned(),
                JsonValue::Bool(true) => "true".to_owned(),
                JsonValue::Bool(false) => "false".to_owned(),
                JsonValue::Number(n) => format!("{}", n),
                JsonValue::String(s) => s.clone(),
                _ => format!("{}", value),
            }
        } else {
            String::default()
        };

        html! {
            <div class="field">
                <label class="label">{ self.name() }</label>
                <div class="control">
                    <input
                        class="input"
                        type="text"
                        placeholder={ self.name() }
                        value=value
                        oninput=&on_input
                        />
                </div>
                { render_description(self.description()) }
            </div>
        }
    }
}

impl RenderableInput for GroupInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();
        html! {
            <div class="field input-group">
                <p class="label">{ self.name() }</p>
                { render_description(self.description()) }
                <div class="input-group-children">
                    { for self
                        .inputs
                        .iter()
                        .map(|input| input.render(&key, &inputs_data, &link))
                    }
                </div>
            </div>
        }
    }
}

impl RenderableInput for NumberInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();
        let key_callback = key.clone();
        let on_input = link.callback(move |input_data: InputData| {
            let number = match &input_data.value {
                s if s.is_empty() => JsonValue::Null,
                s => match s.parse::<JsonNumber>() {
                    Ok(n) => JsonValue::Number(n),
                    Err(_) => JsonValue::String(input_data.value),
                },
            };
            app::Msg::EditedInput(key_callback.clone(), number)
        });

        let value = match inputs_data.get_at(&key) {
            Some(JsonValue::Number(n)) => format!("{}", n),
            Some(JsonValue::String(s)) => s.clone(),
            _ => "".to_string(),
        };

        let min = self
            .min
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let max = self
            .max
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let step = self
            .step
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();

        html! {
            <div class="field">
                <label class="label">{ self.name() }</label>
                <div class="control">
                    <input
                        class="input"
                        type="number"
                        placeholder={ self.name() }
                        value={ value }
                        oninput=&on_input
                        min=min
                        max=max
                        step=step
                        />
                </div>
                { render_description(self.description()) }
            </div>
        }
    }
}

impl RenderableInput for ListInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();

        let list_data = inputs_data.get_at(&key);
        let len = list_data
            .map(JsonValue::as_array)
            .flatten()
            .map(Vec::len)
            .unwrap_or(0);

        let on_resize = |key: Path, new_size| {
            link.callback(move |_: ClickEvent| {
                app::Msg::ListInputSizeChanged(key.clone(), new_size)
            })
        };
        let on_grow = on_resize(key.clone(), len + 1);
        let on_shrink = on_resize(key.clone(), len.saturating_sub(1));

        let render_list_elem = |key_base: Path| {
            let key_base_inner = key_base.clone();
            let on_delete =
                link.callback(move |_: ClickEvent| app::Msg::RemoveAt(key_base_inner.clone()));

            html! {
                <div class="input-group-children">
                    <a class="delete" onclick=on_delete></a>
                    { for self
                        .inputs
                        .iter()
                        .map(|input| input.render(&key_base, &inputs_data, &link))
                    }
                </div>
            }
        };

        html! {
            <div class="field input-group">
                <p class="label">{ self.name() }</p>
                { render_description(self.description()) }

                { for (0..len)
                    .map(|i| &key + &Path(i.to_string()))
                    .map(render_list_elem) }

                <div class="buttons has-addons">
                    <button class="button is-small" onclick=on_grow>
                        <span class="icon is-small">
                            <i class="fas fa-plus"></i>
                        </span>
                    </button>
                    <button class="button is-small" onclick=on_shrink disabled=(len == 0)>
                        <span class="icon is-small">
                            <i class="fas fa-minus"></i>
                        </span>
                    </button>
                </div>
            </div>
        }
    }
}

impl RenderableInput for BooleanInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();

        let key_inner = key.clone();
        let on_click = |b| {
            link.callback(move |_: ClickEvent| {
                app::Msg::EditedInput(key_inner.clone(), JsonValue::Bool(b))
            })
        };

        let checked = match inputs_data.get_at(&key) {
            Some(JsonValue::Null) => false,
            Some(JsonValue::Bool(b)) => *b,
            Some(JsonValue::Number(n)) => n.as_f64() != Some(0.0) && n.as_f64().is_some(),
            Some(JsonValue::String(s)) => s == "true",
            Some(_) => true,
            None => false,
        };
        //let color_class = if checked { "is-success" } else { "is-danger" };
        let id = format!("input_boolean_{}", key);

        html! {
            <div class="field">
                <input id=id name=id type="checkbox" class="switch" checked=checked onclick=on_click(!checked) />
                <label for=id class="label">{ self.name() }</label>
                { render_description(self.description()) }
            </div>
        }
    }
}

fn render_description<T: AsRef<str>>(description: Option<T>) -> Html {
    if let Some(text) = description {
        html! {
            <p class="help">{ text.as_ref() }</p>
        }
    } else {
        html! {}
    }
}
