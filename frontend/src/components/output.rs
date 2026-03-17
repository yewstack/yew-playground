use crate::api::BACKEND_URL;
use crate::{ActionButtonState, ActionButtonStateContext};
use gloo::timers::callback::Interval;
use gloo_net::http::QueryParams;
use std::cell::Cell;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct OutputContainerProps {
    pub value: Rc<str>,
}

#[component]
pub fn OutputContainer(props: &OutputContainerProps) -> Html {
    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();
    let loading = use_state(|| true);
    let elapsed = use_state(|| 0u32);
    let src = use_state(String::new);

    {
        let loading = loading.clone();
        let src = src.clone();
        let elapsed = elapsed.clone();
        use_effect_with(Rc::clone(&props.value), move |value| {
            elapsed.set(0);
            loading.set(true);
            src.set({
                let query = QueryParams::new();
                query.append("code", value);
                format!("{}/run?{}", BACKEND_URL, query)
            });
        })
    };

    {
        let elapsed = elapsed.clone();
        let is_loading = *loading;
        use_effect_with(is_loading, move |is_loading| {
            let _interval = is_loading.then(|| {
                let counter = Rc::new(Cell::new(0u32));
                Interval::new(1_000, {
                    let counter = counter.clone();
                    let elapsed = elapsed.clone();
                    move || {
                        let next = counter.get() + 1;
                        counter.set(next);
                        elapsed.set(next);
                    }
                })
            });
            move || drop(_interval)
        });
    }

    let onload = {
        let loading = loading.clone();
        move |_| {
            loading.set(false);
            action_button_state.dispatch(ActionButtonState::Enabled);
        }
    };

    let classes = classes!(
        "w-full",
        "h-full",
        if *loading { "invisible" } else { "visible" }
    );

    html! {
        <>
            if *loading {
                <div class="h-full bg-gray-600 flex items-center justify-center">
                    <div class="text-gray-200 text-lg flex flex-col items-center gap-2">
                        <span class="animate-spin inline-block w-8 h-8 border-[3px] border-gray-200 border-t-transparent rounded-full"></span>
                        <span>{format!("waiting for the backend service... {}s", *elapsed)}</span>
                    </div>
                </div>
            }
            <iframe src={AttrValue::from((*src).clone())} {onload} class={classes} />
        </>
    }
}
