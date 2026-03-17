use crate::api::BACKEND_URL;
use crate::{ActionButtonState, ActionButtonStateContext};
use gloo::timers::callback::Interval;
use gloo_net::http::{QueryParams, Request};
use std::cell::Cell;
use std::rc::Rc;
use yew::HtmlResult;
use yew::prelude::*;
use yew::suspense::use_future_with;

#[derive(Properties, PartialEq, Eq)]
pub struct OutputContainerProps {
    pub value: Rc<str>,
    pub version: AttrValue,
}

async fn compile(code: &str, version: &str) -> String {
    let query = QueryParams::new();
    query.append("code", code);
    query.append("version", version);
    let url = format!("{}/run?{}", BACKEND_URL, query);

    loop {
        match Request::get(&url).send().await {
            Ok(resp) if resp.status() == 504 => continue,
            Ok(resp) => return resp.text().await.unwrap_or_default(),
            Err(_) => continue,
        }
    }
}

#[component]
pub fn OutputContainer(props: &OutputContainerProps) -> HtmlResult {
    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();

    let value = Rc::clone(&props.value);
    let version = props.version.clone();
    let result = use_future_with((value, version), |deps| async move {
        compile(&deps.0, &deps.1).await
    })?;

    {
        let action_button_state = action_button_state.clone();
        use_effect(move || {
            action_button_state.dispatch(ActionButtonState::Enabled);
        });
    }

    Ok(html! { <iframe srcdoc={(*result).clone()} class="w-full h-full" /> })
}

#[component]
pub fn CompileTimer() -> Html {
    let elapsed = use_state(|| 0u32);

    {
        let elapsed = elapsed.clone();
        use_effect_with((), move |_| {
            let counter = Rc::new(Cell::new(0u32));
            let interval = Interval::new(1_000, {
                let counter = counter.clone();
                let elapsed = elapsed.clone();
                move || {
                    let next = counter.get() + 1;
                    counter.set(next);
                    elapsed.set(next);
                }
            });
            move || drop(interval)
        });
    }

    html! {
        <div class="h-full bg-gray-600 flex items-center justify-center">
            <div class="text-gray-200 text-lg flex flex-col items-center gap-2">
                <span class="animate-spin inline-block w-8 h-8 border-[3px] border-gray-200 border-t-transparent rounded-full"></span>
                <span>{format!("waiting for the backend service... {}s", *elapsed)}</span>
            </div>
        </div>
    }
}
