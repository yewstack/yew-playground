use anyhow::Result;
use gloo::console::{console, log};
use monaco::api::TextModel;
use monaco::sys::editor::IModelContentChangedEvent;
use monaco::yew::CodeEditor;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme};
use serde::Deserialize;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew::suspense::use_future_with_deps;
use yew::HtmlResult;
use yew_router::hooks::use_location;

const BASE_CONTENT: &str = r#"
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! { "hello world" }
}

fn main() {
    yew::start_app::<App>();
}
"#;

#[derive(Debug, Deserialize, PartialEq)]
struct Query {
    shared: Option<String>,
}

#[derive(Clone)]
struct TextContent(Rc<Option<Result<String>>>);

impl Deref for TextContent {
    type Target = Option<Result<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TextContent {
    fn new(val: Option<Result<String>>) -> Self {
        Self(Rc::new(val))
    }
}

impl PartialEq for TextContent {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
        // other.0.as_ref().unwrap().as_ref().unwrap()
        // let o = match (&*self.0, &*other.0) {
        //     (Some(Ok(value_self)), Some(Ok(value_other))) => { *value_self == *value_other },
        //     (None, None) => { true },
        //     (Some(Err(err_self)), Some(Err(err_other))) => { err_self.to_string() == err_other.to_string() },
        //     _ => false
        // };
        // console!(o.to_string());
        // o
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
    pub oninput: Callback<(IModelContentChangedEvent, String)>
}

#[function_component]
pub fn Editor(props: &EditorProps) -> HtmlResult {
    let location = use_location().unwrap();
    let query = location.query::<Query>().unwrap();
    log!(query
        .shared
        .as_ref()
        .map(JsValue::from)
        .unwrap_or(JsValue::NULL));

    let text_content = use_future_with_deps(
        |query| async move {
            TextContent::new(query.shared.as_ref().map(|text| {
                // todo actually fetch things
                Ok::<_, anyhow::Error>(text.to_string())
            }))
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
                let disposable = modal.on_did_change_content(move |c: IModelContentChangedEvent| {
                    cb.emit((c, modal2.get_value()));
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
