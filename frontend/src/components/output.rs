use std::rc::Rc;
use gloo::timers::callback::Timeout;

use crate::api::{self, run::Response};
use crate::{ActionButtonState, ActionButtonStateContext};
use js_sys::ArrayBuffer;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlDocument, HtmlIFrameElement};
use yew::prelude::*;
use yew::suspense::{use_future_with_deps, Suspense};

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

const IFRAME_ID: &str = "output-frame";

fn load_into_iframe(iframe: HtmlIFrameElement, index_html: String, buf: ArrayBuffer) {
    let window = iframe.content_window().unwrap();
    let doc = window
        .document()
        .unwrap()
        .unchecked_into::<HtmlDocument>();
    doc.open().unwrap();
    doc.write(&JsValue::from(index_html).into()).unwrap();
    doc.close().unwrap();
    // chrome needs this
    let timeout = Timeout::new(0, move || {
        let iframe: HtmlIFrameElement = gloo::utils::document().get_element_by_id(IFRAME_ID).unwrap().unchecked_into();
        let window = iframe.content_window().unwrap();
        let init = js_sys::Reflect::get(window.as_ref(), &"init".into()).unwrap();
        let init = init.dyn_into::<js_sys::Function>().unwrap();
        init.call1(&window.into(), &JsValue::from(buf)).unwrap();
    });
    // we need this timeout to finish to completion
    std::mem::forget(timeout);
}

fn into_array_buf(slice: &[u8]) -> ArrayBuffer {
    let uint8 = js_sys::Uint8Array::new_with_length(slice.len().try_into().unwrap());
    for (i, val) in slice.iter().enumerate() {
        uint8.set_index(i.try_into().unwrap(), *val);
    }
    uint8.buffer()
}

#[derive(Properties, PartialEq)]
pub struct OutputContainerProps {
    pub value: Rc<str>,
}

#[function_component]
pub fn OutputContainer(props: &OutputContainerProps) -> Html {
    let fallback = html! { <div class="h-full bg-gray-600">{"Loading"}</div> };
    html! {
        <Suspense {fallback}>
            <Output value={Rc::clone(&props.value)} />
        </Suspense>
    }
}

#[function_component]
pub fn Output(props: &OutputContainerProps) -> HtmlResult {
    let resp = use_future_with_deps(
        |value| async move { api::run(&*value).await },
        Rc::clone(&props.value),
    )?;

    let iframe_ref = use_node_ref();
    let action_button_state = use_context::<ActionButtonStateContext>().unwrap();

    let onload = {
        let iframe_ref = iframe_ref.clone();

        move |_| {
            match &*resp {
                Ok(Response::Render {
                    js,
                    wasm,
                    index_html: _,
                }) => {
                    let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();

                    let index_html = INDEX_HTML.replace("/*JS_GOES_HERE*/", js);
                    let buf = into_array_buf(wasm);
                    load_into_iframe(iframe, index_html, buf);
                }
                Ok(Response::CompileError(data)) => {
                    let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();

                    let document = iframe.content_document().unwrap();
                    document
                        .body()
                        .unwrap()
                        .set_inner_html(&format!("<pre><code>{data}</pre></code>"));
                }
                Err(e) => gloo::console::error!(e.to_string()),
            };
            action_button_state.dispatch(ActionButtonState::Enabled);
        }
    };

    Ok(html! {
        <iframe id={IFRAME_ID} ref={iframe_ref.clone()} {onload} class="w-full h-full border-t-[10px] border-gray-400 border-solid" />
    })
}
