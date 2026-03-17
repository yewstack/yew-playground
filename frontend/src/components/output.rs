use crate::api::BACKEND_URL;
use crate::{ActionButtonState, ActionButtonStateContext};
use gloo::timers::callback::Interval;
use gloo_net::http::{QueryParams, Request};
use std::cell::Cell;
use std::rc::Rc;
use yew::HtmlResult;
use yew::prelude::*;
use yew::suspense::use_future_with;

#[derive(Clone, PartialEq)]
enum CompileResult {
    Done(String),
    ColdStartTimeout,
}

#[derive(Properties, PartialEq, Eq)]
pub struct OutputContainerProps {
    pub value: Rc<str>,
}

#[component]
pub fn OutputContainer(props: &OutputContainerProps) -> HtmlResult {
    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();

    let result = use_future_with(Rc::clone(&props.value), |value| async move {
        let query = QueryParams::new();
        query.append("code", &value);
        let url = format!("{}/run?{}", BACKEND_URL, query);

        match Request::get(&url).send().await {
            Ok(resp) if resp.status() == 504 => CompileResult::ColdStartTimeout,
            Ok(resp) => CompileResult::Done(resp.text().await.unwrap_or_default()),
            Err(_) => CompileResult::ColdStartTimeout,
        }
    })?;

    {
        let action_button_state = action_button_state.clone();
        use_effect(move || {
            action_button_state.dispatch(ActionButtonState::Enabled);
        });
    }

    Ok(match &*result {
        CompileResult::Done(html_content) => {
            html! { <iframe srcdoc={html_content.clone()} class="w-full h-full" /> }
        }
        CompileResult::ColdStartTimeout => {
            html! {
                <div class="h-full bg-gray-600 flex items-center justify-center">
                    <div class="text-gray-200 flex flex-col items-center gap-3 max-w-md text-center">
                        <span class="text-xl font-semibold">{"Backend service is waking up"}</span>
                        <span class="text-gray-400">{"The service was asleep and timed out on the first request. It should be ready now."}</span>
                        <span class="text-gray-300">{"Hit Run again."}</span>
                    </div>
                </div>
            }
        }
    })
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
