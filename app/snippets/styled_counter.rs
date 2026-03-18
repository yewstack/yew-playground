use stylist::yew::styled_component;
use tracing::info;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use yew::prelude::*;

#[styled_component]
fn App() -> Html {
    let count = use_state(|| 0i32);

    html! {
        <div class={css!(
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            font-family: system-ui, sans-serif;
            background: #1a1a2e;
            color: #eee;
        )}>
            <h1 class={css!(font-size: 3rem; margin-bottom: 1.5rem;)}>
                {*count}
            </h1>
            <div class={css!(display: flex; gap: 1rem;)}>
                <button
                    onclick={
                        let count = count.clone();
                        move |_| {
                            info!(value = *count - 1, "decrement");
                            count.set(*count - 1);
                        }
                    }
                    class={css!(
                        padding: 0.75rem 1.5rem;
                        font-size: 1.25rem;
                        border: none;
                        border-radius: 0.5rem;
                        background: #e94560;
                        color: white;
                        cursor: pointer;
                    )}
                >{"-"}</button>
                <button
                    onclick={move |_| {
                        info!(value = *count + 1, "increment");
                        count.set(*count + 1);
                    }}
                    class={css!(
                        padding: 0.75rem 1.5rem;
                        font-size: 1.25rem;
                        border: none;
                        border-radius: 0.5rem;
                        background: #0f3460;
                        color: white;
                        cursor: pointer;
                    )}
                >{"+"}</button>
            </div>
            <p class={css!(margin-top: 1rem; color: #888; font-size: 0.85rem;)}>
                {"Open the browser console to see tracing output"}
            </p>
        </div>
    }
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(tracing_web::MakeWebConsoleWriter::new().with_pretty_level())
        .with_level(false);
    let perf_layer = tracing_web::performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<App>::new().render();
}
