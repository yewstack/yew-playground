pub mod query;

use yew::virtual_dom::VNode;
use yew::Classes;

pub fn html_to_element(html: &str, classes: Option<Classes>) -> VNode {
    let div = gloo::utils::document().create_element("div").unwrap();
    div.set_inner_html(html);
    let node = div.children().item(0).unwrap();
    if let Some(classes) = classes {
        node.set_attribute("class", &classes.to_string())
            .expect("failed to set classes");
    }
    VNode::VRef(node.into())
}
