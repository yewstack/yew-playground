use std::rc::Rc;
use gloo::console::log;

use js_sys::ArrayBuffer;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlDocument, HtmlIFrameElement};
use yew::prelude::*;
use yew::suspense::{Suspense, use_future_with_deps};
use crate::api::{self, run::Response};

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

fn load_into_iframe(iframe: HtmlIFrameElement, index_html: String, buf: &ArrayBuffer) {
    log!(&iframe);
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
    log!(&init);
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

#[derive(Properties, PartialEq)]
pub struct OutputContainerProps {
    pub value: Rc<str>
}


#[function_component]
pub fn OutputContainer(props: &OutputContainerProps) -> Html {
    let fallback= html! { "loading"};
    html! {
        <div>
            <Suspense {fallback}>
                <Output value={Rc::clone(&props.value)} />
            </Suspense>
        </div>
    }
}

#[function_component]
pub fn Output(props: &OutputContainerProps) -> HtmlResult {
    let resp = use_future_with_deps(|value| async move {
        api::run(&*value).await.unwrap()
    }, Rc::clone(&props.value))?;

    let iframe_ref = use_node_ref();

    let onload = {
        let iframe_ref = iframe_ref.clone();
        let resp = Rc::clone(&resp);

        move |_| {
            match &*resp {
                Response::Render(data) => {
                    let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();

                    let index_html = INDEX_HTML.replace("/*JS_GOES_HERE*/", &data.js);
                    let buf = into_array_buf(&data.wasm);
                    load_into_iframe(iframe, index_html, &buf);
                }
                Response::CompileError(data) => {
                    let iframe = iframe_ref.cast::<HtmlIFrameElement>().unwrap();

                    let document = iframe.content_document().unwrap();
                    document
                        .body()
                        .unwrap()
                        .set_inner_html(&format!("<pre><code>{data}</pre></code>"));
                }
            };
        }
    };

    Ok(html! {
        <iframe ref={iframe_ref.clone()} {onload} />
    })
}
