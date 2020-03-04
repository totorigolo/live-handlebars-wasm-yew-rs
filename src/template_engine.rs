use crate::scenario::Template;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;

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
    pub fn with_template(template: &Template) -> Self {
        let mut s = Self::new_uninit();
        s.set_template(template);
        s
    }

    #[allow(unused)]
    pub fn set_template(&mut self, template: &Template) -> Result<()> {
        match template {
            Template::StringTemplate(s) => self.inner.register_template_string("t", s),
            Template::StringListTemplate(ls) => {
                self.inner.register_template_string("t", ls.join("\n"))
            }
        }
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
