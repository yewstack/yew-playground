
use yew::prelude::*;

#[component]
fn App() -> Html {
    html! { "hello world" }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
