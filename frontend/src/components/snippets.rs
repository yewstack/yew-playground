use crate::snippets::snippets_for;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SnippetPickerProps {
    pub version: AttrValue,
    pub on_select: Callback<&'static str>,
}

#[component]
pub fn SnippetPicker(props: &SnippetPickerProps) -> Html {
    let open = use_state(|| false);
    let snippets = snippets_for(&props.version);

    html! {
        <div class="relative">
            <button
                onclick={{
                    let open = open.clone();
                    move |_: MouseEvent| open.set(!*open)
                }}
                class="p-3 text-sm cursor-pointer bg-gray-800 rounded-md shadow-lg text-gray-400 hover:bg-gray-900 flex items-center gap-1"
            >
                {"Examples"}
                <span class="text-xs">{if *open { "▲" } else { "▼" }}</span>
            </button>
            if *open {
                <div class="absolute left-0 top-full mt-1 z-50 bg-gray-800 border border-gray-600 rounded-md shadow-xl py-1 min-w-[160px]">
                    for snippet in snippets {
                        <button
                            onclick={{
                                let on_select = props.on_select.clone();
                                let open = open.clone();
                                let code = snippet.code;
                                move |_: MouseEvent| {
                                    on_select.emit(code);
                                    open.set(false);
                                }
                            }}
                            class="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-gray-700 cursor-pointer"
                        >
                            {snippet.name}
                        </button>
                    }
                </div>
            }
        </div>
    }
}
