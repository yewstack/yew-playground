use std::rc::Rc;

use js_sys::ArrayBuffer;
use monaco::api::TextModel;
use monaco::{api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlDocument, HtmlIFrameElement, RequestMode};
use yew::prelude::*;

const INDEX_HTML: &str = r#"
<!doctype html>
<html lang="en">
<head>
<meta charset="UTF-8">
 <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
             <meta http-equiv="X-UA-Compatible" content="ie=edge">
 <title>Document</title>
</head>
<body>
    <script type="module">
        /*JS_GOES_HERE*/
        window.init = (arrayBuf) => init(arrayBuf)
    </script>
</body>
</html>
"#;

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

fn get_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_builtin_theme(BuiltinTheme::VsDark)
        .with_scroll_beyond_last_line(false)
        .with_automatic_layout(true)
}

fn load_into_iframe(iframe: HtmlIFrameElement, index_html: String, buf: &ArrayBuffer) {
    let window = iframe.content_window().unwrap();
    let doc = window
        .document()
        .unwrap()
        .dyn_into::<HtmlDocument>()
        .unwrap();
    doc.open().unwrap();
    doc.write(&JsValue::from(index_html).into()).unwrap();
    doc.close().unwrap();
    let init = js_sys::Reflect::get(&window.into(), &"init".into()).unwrap();
    let init = init.dyn_into::<js_sys::Function>().unwrap();
    gloo::console::log!(&init);
    let window = iframe.content_window().unwrap();
    init.call1(&window.into(), &JsValue::from(buf)).unwrap();
}

fn into_array_buf(slice: &[u8]) -> ArrayBuffer {
    let uint8 = js_sys::Uint8Array::new_with_length(slice.len().try_into().unwrap());
    for (i, val) in slice.iter().enumerate() {
        uint8.set_index(i.try_into().unwrap(), *val);
    }
    uint8.buffer()
}

#[derive(Serialize)]
struct RunPayload {
    main_contents: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct RunResponse {
    index_html: String,
    js: String,
    wasm: Vec<u8>,
}

#[derive(PartialEq)]
enum State {
    Loading,
    Loaded(RunResponse),
    Error(String),
}

#[function_component]
fn App() -> Html {
    let iframe_ref = use_node_ref();
    let data = use_state(|| State::Loading);

    let modal = use_memo(
        |_| TextModel::create(BASE_CONTENT.trim(), Some("rust"), None).unwrap(),
        (),
    );
    let on_run_click = {
        let modal = modal.clone();
        let data = data.clone();
        move |_| {
            let value = modal.get_value();

            let data = data.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let payload = RunPayload {
                    main_contents: value,
                };
                let resp = reqwasm::http::Request::post("http://localhost:3000/run")
                    .body(serde_json::to_string(&payload).unwrap())
                    .header("Content-Type", "application/json")
                    .send()
                    .await
                    .unwrap();
                if resp.status() == 200 {
                    let bin = resp.binary().await.unwrap();
                    let resp = bson::from_slice::<RunResponse>(&bin).unwrap();
                    data.set(State::Loaded(resp));
                } else {
                    let err = resp.text().await.unwrap_or_else(|e| e.to_string());
                    data.set(State::Error(err));
                }
            });
        }
    };

    {
        let iframe_ref = iframe_ref.clone();
        use_effect_with_deps(
            move |data| {
                let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();
                match &**data {
                    State::Loaded(data) => {
                        let index_html = INDEX_HTML.replace("/*JS_GOES_HERE*/", &data.js);
                        let buf = into_array_buf(&data.wasm);
                        load_into_iframe(iframe, index_html, &buf);
                    }
                    State::Error(e) => {
                        let document = iframe.content_document().unwrap();
                        document
                            .body()
                            .unwrap()
                            .set_inner_html(&format!("<pre><code>{e}</pre></code>"));
                    }
                    _ => {
                        gloo::console::log!("loading")
                    }
                }
                || ()
            },
            data,
        );
    }

    html! {
        <>
            <header>
                <button onclick={on_run_click}>{"run"}</button>
            </header>
            <CodeEditor options={Rc::new(get_options())} classes="the-editor" model={Some((*modal).clone())} />
            <iframe ref={iframe_ref.clone()} />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
