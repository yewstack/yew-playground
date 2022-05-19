#![feature(const_option_ext)]

mod api;
mod output;
mod macros;
mod utils;
mod app;

use yew::prelude::*;
use std::rc::Rc;
use app::App;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActionButtonState {
    Enabled,
    Disabled
}

#[derive(Debug, PartialEq, Clone)]
pub struct ActionButtonStateReducible {
    state: ActionButtonState,
}

impl ActionButtonStateReducible {
    pub fn disabled(&self) -> bool {
        matches!(self.state, ActionButtonState::Disabled)
    }
}

impl Reducible for ActionButtonStateReducible {
    type Action = ActionButtonState;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        ActionButtonStateReducible { state: action }.into()
    }
}

pub type ActionButtonStateContext = UseReducerHandle<ActionButtonStateReducible>;

#[function_component]
fn Root() -> Html {
    let msg = use_reducer(|| ActionButtonStateReducible {
        state: ActionButtonState::Enabled
    });

    html! {
        <ContextProvider<ActionButtonStateContext> context={msg}>
            <App />
        </ContextProvider<ActionButtonStateContext>>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}
