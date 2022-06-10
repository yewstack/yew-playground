
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! { "hello world" }
}

fn main() {
    yew::start_app::<App>();
}
