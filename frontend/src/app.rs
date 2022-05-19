use crate::output::OutputContainer;
use crate::{icon, ActionButtonState, ActionButtonStateContext};
use monaco::api::TextModel;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor};
use std::rc::Rc;
use yew::prelude::*;

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
pub fn App() -> Html {
    let data = use_state(|| None);

    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();

    let modal = use_memo(
        |_| TextModel::create(BASE_CONTENT.trim(), Some("rust"), None).unwrap(),
        (),
    );

    let on_run_click = {
        let modal = modal.clone();
        let data = data.clone();
        let action_button_state = action_button_state.clone();
        move |_| {
            let value = modal.get_value();
            let data = data.clone();
            data.set(Some(Rc::from(value)));
            action_button_state.dispatch(ActionButtonState::Disabled);
        }
    };

    let mut classes = Classes::from(
        "p-3 text-center shadow-lg bg-gray-800 rounded-md flex gap-2 \
    transition duration-200 ease-in-out disabled:cursor-not-allowed disabled:bg-gray-700",
    );
    if !action_button_state.disabled() {
        classes.push("hover:bg-gray-900")
    }

    let template_rows = if data.is_some() {
        "grid-template-rows: 1fr 1fr"
    } else {
        "grid-template-rows: 1fr "
    };

    html! {
        <div class="flex flex-col h-screen">
            <header class="bg-gray-700 p-3">
                <button onclick={on_run_click} disabled={action_button_state.disabled()} class={classes}>{icon!("play_arrow", classes!("fill-gray-200"))} {"Run"}</button>
            </header>
            <div class="grid h-full" style={template_rows}>
                <CodeEditor options={get_options().to_sys_options()} classes="the-editor h-full min-h-0" model={Some((*modal).clone())} />
                <div class="w-full h-full min-h-0">
                    if let Some(ref data) = *data {
                        <OutputContainer value={data} />
                    } else {
                        // <div class="h-full bg-gray-600" />
                    }
                </div>
            </div>
        </div>
    }
}
