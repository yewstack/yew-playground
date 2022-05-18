#![feature(const_option_ext)]

mod api;
mod output;

use monaco::api::TextModel;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor};
use yew::prelude::*;
use output::OutputContainer;
use std::rc::Rc;

const BASE_CONTENT: &str = r#"
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! { "hello world" }
}

fn main() {
    yew::start_app::<App>();
}
"#;

fn get_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_builtin_theme(BuiltinTheme::VsDark)
        .with_scroll_beyond_last_line(false)
        .with_automatic_layout(true)
}

#[function_component]
fn App() -> Html {
    let data = use_state(|| None);

    let modal = use_memo(
        |_| TextModel::create(BASE_CONTENT.trim(), Some("rust"), None).unwrap(),
        (),
    );

    let on_run_click = {
        let modal = modal.clone();
        let data = data.clone();
        move |_| {
            let value = modal.get_value();
            let data = data.clone();
            data.set(Some(Rc::from(value)))
        }
    };

    html! {
        <>
            <header>
                <button onclick={on_run_click} class="p-3 text-center shadow-lg">{"Run"}</button>
            </header>
            <CodeEditor options={get_options().to_sys_options()} classes="the-editor" model={Some((*modal).clone())} />
            if let Some(ref data) = *data {
                <OutputContainer value={data} />
            }
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
