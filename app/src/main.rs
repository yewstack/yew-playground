#![allow(unused)]
extern crate r#yew;
fn main() {
use yew::prelude::*;

#[function_component(App)]
fn HelloWorld() -> Html {
    html! { "Hello world" }
}

yew::start_app::<App>();
}