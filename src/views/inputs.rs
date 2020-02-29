use crate::{for_all_inputtypes_variants, inputs::*, prelude::*, InputsData, Path};

type AppComponentLink = yew::ComponentLink<crate::app::Model>;

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
        let on_input = link.callback(move |input_data: InputData| {
            crate::app::Msg::EditedInput(key.clone(), JsonValue::String(input_data.value))
        });

        let value = if let Some(value) = inputs_data.get_at(&self.key()) {
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
                        value={ value }
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
            <>
                <p>{ self.name() }</p>
                { render_description(self.description()) }
                { for self
                    .inputs
                    .iter()
                    .map(|input| input.render(&key, &inputs_data, &link))
                }
            </>
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
            crate::app::Msg::EditedInput(key_callback.clone(), number)
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
                crate::app::Msg::ListInputSizeChanged(key.clone(), new_size)
            })
        };
        let on_grow = on_resize(key.clone(), len + 1);
        let on_shrink = on_resize(key.clone(), len.saturating_sub(1));

        fn render_list_elem(
            inputs: &[InputTypes],
            key: Path,
            inputs_data: &InputsData,
            link: &AppComponentLink,
        ) -> Html {
            html! {
                <div>
                    { for inputs
                        .iter()
                        .map(|input| input.render(&key, &inputs_data, &link))
                    }
                </div>
            }
        }

        html! {
            <>
                <p>{ self.name() }</p>
                <button class="button" onclick=on_grow>{ "Add elem" }</button>
                <button class="button" onclick=on_shrink>{ "Remove elem" }</button>
                { render_description(self.description()) }
                { for (0..len)
                    .map(|i| &key + &Path(i.to_string()))
                    .map(|path| render_list_elem(&self.inputs, path, &inputs_data, &link)) }
            </>
        }
    }
}

impl RenderableInput for BooleanInput {
    fn render(&self, key_base: &Path, inputs_data: &InputsData, link: &AppComponentLink) -> Html {
        let key = key_base + self.key();
        let on_input = link.callback(move |input_data: InputData| {
            crate::app::Msg::EditedInput(key.clone(), JsonValue::Bool(input_data.value == "true"))
        });

        let checked = match inputs_data.get_at(&self.key()) {
            Some(JsonValue::Null) => false,
            Some(JsonValue::Bool(b)) => *b,
            Some(JsonValue::Number(n)) => n.as_f64() != Some(0.0) && n.as_f64().is_some(),
            Some(JsonValue::String(s)) => s == "true",
            Some(_) => true,
            None => false,
        };

        html! {
            <div class="field">
                <label class="checkbox">
                    <input
                        type="checkbox"
                        value=if checked { "false" } else { "true" }
                        oninput=&on_input
                        checked=checked
                        />
                    { self.name() }
                </label>
                <label class="label">{ self.name() }</label>
                { render_description(self.description()) }
            </div>
        }
    }
}

fn render_description<T: AsRef<str>>(description: Option<T>) -> Html {
    if let Some(text) = description {
        html! {
            <p class="description">{ text.as_ref() }</p>
        }
    } else {
        html! {}
    }
}
