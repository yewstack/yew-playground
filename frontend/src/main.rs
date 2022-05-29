#![feature(const_option_ext)]

mod api;
mod app;
mod components;
mod macros;
mod utils;

use app::App;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActionButtonState {
    Enabled,
    Disabled,
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
        state: ActionButtonState::Enabled,
    });

    html! {
        <BrowserRouter>
            <ContextProvider<ActionButtonStateContext> context={msg}>
                <App />
            </ContextProvider<ActionButtonStateContext>>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}
