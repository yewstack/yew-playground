use crate::{ActionButtonState, ActionButtonStateContext};
use crate::api::BACKEND_URL;
use gloo_net::http::QueryParams;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct OutputContainerProps {
    pub value: Rc<str>,
}

#[function_component]
pub fn OutputContainer(props: &OutputContainerProps) -> Html {
    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();
    let loading = use_state(|| true);
    let src = use_state(|| AttrValue::from("about:black"));

    {
        let loading = loading.clone();
        let src = src.clone();
        use_effect_with_deps(
            move |value| {
                src.set({
                    let query = QueryParams::new();
                    query.append("code", value);
                    AttrValue::from(format!("{}/run?{}", BACKEND_URL, query))
                });
                loading.set(false);
            },
            Rc::clone(&props.value),
        )
    };

    let fallback = html! { <div class="h-full bg-gray-600">{"Loading"}</div> };

    let onload = move |_| {
        action_button_state.dispatch(ActionButtonState::Enabled);
    };
    let classes = classes!(
        "w-full",
        "h-full",
        "border-t-[10px]",
        "border-gray-400",
        "border-solid",
        if *loading { "invisible" } else { "visible" }
    );
    html! {
        <>
            if *loading {
                {fallback}
            }
            <iframe src={AttrValue::clone(&*src)} {onload} class={classes} />
        </>
    }
}
