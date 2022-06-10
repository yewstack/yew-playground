use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! { "hello pastify" }
}

fn main() {
    yew::start_app::<App>();
}
