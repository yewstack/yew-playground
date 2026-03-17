use std::collections::BTreeMap;

use yew::prelude::*;

type DepMap = BTreeMap<String, String>;

fn parse_deps(json: &str) -> DepMap {
    serde_json::from_str(json).unwrap_or_default()
}

#[derive(Properties, PartialEq)]
pub struct CratesPanelProps {
    pub version: AttrValue,
}

#[component]
pub fn CratesPanel(props: &CratesPanelProps) -> Html {
    let open = use_state(|| false);

    let deps = if props.version == "next" {
        parse_deps(env!("APP_DEPS_NEXT"))
    } else {
        parse_deps(env!("APP_DEPS_STABLE"))
    };

    html! {
        <div class="relative">
            <button
                onclick={{
                    let open = open.clone();
                    move |_: MouseEvent| open.set(!*open)
                }}
                class="p-3 text-sm cursor-pointer bg-gray-800 rounded-md shadow-lg text-gray-400 hover:bg-gray-900 flex items-center gap-1"
            >
                {"Crates"}
                <span class="text-xs">{if *open { "▲" } else { "▼" }}</span>
            </button>
            if *open {
                <div class="absolute right-0 top-full mt-1 z-50 bg-gray-800 border border-gray-600 rounded-md shadow-xl p-3 min-w-[200px]">
                    <div class="text-gray-400 text-xs font-semibold mb-2 uppercase tracking-wider">{"Available Crates"}</div>
                    <div class="flex flex-col gap-1">
                        for (name, ver) in deps {
                            <div class="flex justify-between gap-4 text-sm">
                                <span class="text-gray-200 font-mono">{name}</span>
                                <span class="text-gray-500 font-mono">{ver}</span>
                            </div>
                        }
                    </div>
                </div>
            }
        </div>
    }
}
