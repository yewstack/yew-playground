use crate::rc_type;
use crate::utils::query::use_query;
use anyhow::Result;
use gloo::console::log;
use monaco::api::TextModel;
use monaco::yew::CodeEditor;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme};
use std::rc::Rc;
use yew::HtmlResult;
use yew::prelude::*;
use yew::suspense::use_future_with;

const BASE_CONTENT: &str = crate::snippets::STABLE_SNIPPETS[0].code;

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
    #[prop_or_default]
    pub snippet_code: Option<AttrValue>,
}

#[component]
pub fn Editor(props: &EditorProps) -> HtmlResult {
    let query = use_query().unwrap();

    let text_content = use_future_with(query, |query| async move {
        if let Some(code) = &query.code {
            return TextContent::new_with_string(code.to_string());
        }

        let shared = match &query.shared {
            Some(text) => Some(
                crate::api::share::get(text)
                    .await
                    .map(|paste| paste.fields.into_content()),
            ),
            None => None,
        };
        TextContent::new(shared)
    })?;

    // Extract the text content and wrap in Rc for use as memo dependency
    let content_rc = (*text_content).clone();

    let modal = use_memo(content_rc, |text_content| {
        let text = match &**text_content {
            Some(Ok(text)) => text,
            Some(Err(e)) => panic!("failed to fetch data: {}", e),
            None => BASE_CONTENT,
        };
        TextModel::create(text, Some("rust"), None).unwrap()
    });

    {
        let modal = modal.clone();
        let cb = props.oninput.clone();
        use_effect_with(modal, move |modal| {
            log!("we got called");
            let modal2 = modal.clone();
            cb.emit(modal2.get_value());
            let disposable = modal.on_did_change_content(move |_| {
                cb.emit(modal2.get_value());
            });

            move || drop(disposable)
        })
    }

    {
        let modal = modal.clone();
        let snippet_code = props.snippet_code.clone();
        use_effect_with(snippet_code, move |code| {
            if let Some(code) = code {
                modal.set_value(code);
            }
        });
    }

    Ok(html! {
        <CodeEditor options={get_options().to_sys_options()} classes="the-editor h-full min-h-0" model={Some((*modal).clone())} />
    })
}
