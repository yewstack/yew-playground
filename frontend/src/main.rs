mod api;
mod app;
mod components;
mod macros;
mod utils;
use tracing_web::{MakeConsoleWriter, performance_layer};
use tracing_subscriber::fmt::format::{FmtSpan, Pretty};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;

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
    let msg = use_reducer_eq(|| ActionButtonStateReducible {
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
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter)
        .with_span_events(FmtSpan::ACTIVE);
    let perf_layer = performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<Root>::new().render();
}
