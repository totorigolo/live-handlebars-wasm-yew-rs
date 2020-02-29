use wasm_live_handlebars::app;

fn main() {
    web_logger::init();
    yew::start_app::<app::Model>();
}
