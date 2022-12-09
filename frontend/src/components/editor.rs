use crate::rc_type;
use crate::utils::query::use_query;
use anyhow::Result;
use gloo::console::log;
use monaco::api::TextModel;
use monaco::yew::CodeEditor;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme};
use std::rc::Rc;
use yew::prelude::*;
use yew::suspense::use_future_with_deps;
use yew::HtmlResult;

const BASE_CONTENT: &str = r#"
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! { "hello world" }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
"#;

rc_type!(TextContent => Option<Result<String>>);

impl TextContent {
    fn new(val: Option<Result<String>>) -> Self {
        Self(Rc::new(val))
    }
    fn new_with_string(text: String) -> Self {
        Self::new(Some(Ok(text)))
    }
}

fn get_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_builtin_theme(BuiltinTheme::VsDark)
        .with_scroll_beyond_last_line(false)
        .with_automatic_layout(true)
}

#[derive(PartialEq, Properties)]
pub struct EditorProps {
    pub oninput: Callback<String>,
}

#[function_component]
pub fn Editor(props: &EditorProps) -> HtmlResult {
    let query = use_query().unwrap();

    let text_content = use_future_with_deps(
        |query| async move {
            if let Some(code) = &query.code {
                return TextContent::new_with_string(code.to_string());
            }

            let shared = match &query.shared {
                Some(text) => Some(
                    crate::api::share::get(text)
                        .await
                        .map_err(anyhow::Error::from)
                        .map(|paste| paste.fields.into_content()),
                ),
                None => None,
            };
            TextContent::new(shared)
        },
        query,
    )?;

    let modal = use_memo(
        |text_content| {
            let text = match &**text_content {
                Some(Ok(text)) => text,
                Some(Err(e)) => panic!("failed to fetch data: {}", e),
                None => BASE_CONTENT,
            };
            TextModel::create(text, Some("rust"), None).unwrap()
        },
        text_content.clone(),
    );

    {
        let modal = modal.clone();
        let cb = props.oninput.clone();
        use_effect_with_deps(
            move |modal| {
                log!("we got called");
                let modal2 = modal.clone();
                cb.emit(modal2.get_value());
                let disposable = modal.on_did_change_content(move |_| {
                    cb.emit(modal2.get_value());
                });

                move || drop(disposable)
            },
            modal,
        )
    }

    Ok(html! {
        <CodeEditor options={get_options().to_sys_options()} classes="the-editor h-full min-h-0" model={Some((*modal).clone())} />
    })
}
