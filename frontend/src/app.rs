use crate::components::editor::Editor;
use crate::components::output::OutputContainer;
use crate::utils::query::Query;
use crate::{icon, ActionButtonState, ActionButtonStateContext};
use gloo::history::{BrowserHistory, History};
use std::rc::Rc;
use yew::prelude::*;
use yew::suspense::Suspense;

#[function_component]
pub fn App() -> Html {
    let editor_contents = use_mut_ref(String::new);
    let data = use_state(|| None);

    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();

    let on_run_click = {
        let action_button_state = action_button_state.clone();
        let editor_contents = editor_contents.clone();
        let data = data.clone();
        move |_| {
            data.set(Some(Rc::from(editor_contents.as_ref().borrow().as_str())));
            action_button_state.dispatch(ActionButtonState::Disabled);
        }
    };

    let on_share_click = {
        let editor_contents = editor_contents.clone();
        let action_button_state = action_button_state.clone();
        move |_| {
            let action_button_state = action_button_state.clone();
            let editor_contents = editor_contents.clone();

            // https://tenor.com/view/this-is-fine-fire-house-burning-okay-gif-5263684
            // this is fine because no one can do anuthing with the refcell while we holding it
            // alternate is clone, which is expensive
            #[allow(clippy::await_holding_refcell_ref)]
            wasm_bindgen_futures::spawn_local(async move {
                action_button_state.dispatch(ActionButtonState::Disabled);
                let history = BrowserHistory::new();

                let content = editor_contents.as_ref().borrow();
                let content = content.as_str();
                let paste = crate::api::share::create(content).await.expect("fucked up");
                let id = paste.id();
                let query = Query {
                    shared: Some(id),
                    code: None,
                };
                history
                    .push_with_query("/", query)
                    .expect("failed to navigate");
                action_button_state.dispatch(ActionButtonState::Enabled);
            })
        }
    };

    let oninput = {
        move |v| {
            *editor_contents.as_ref().borrow_mut() = v;
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
            <header class="bg-gray-700 p-3 flex justify-between">
                <button onclick={on_run_click} disabled={action_button_state.disabled()} class={classes.clone()}>{icon!("play_arrow", classes!("fill-gray-200"))} {"Run"}</button>

                <div>
                    <button onclick={on_share_click} disabled={action_button_state.disabled()} class={classes}>{icon!("share", classes!("fill-gray-200"))} {"Share"}</button>
                </div>
            </header>
            <div class="grid h-full" style={template_rows}>
                <Suspense fallback={{html! {"loading..."}}}>
                    <Editor {oninput} />
                </Suspense>
                <div class="w-full h-full min-h-0">
                    if let Some(ref data) = *data {
                        <OutputContainer value={data} />
                    }
                </div>
            </div>
        </div>
    }
}
