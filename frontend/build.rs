use std::collections::BTreeMap;

const HIDDEN_DEPS: &[&str] = &[
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "web-sys",
    "js-sys",
    "getrandom",
    "implicit-clone",
];

fn extract_deps(toml: &toml::Value) -> String {
    let deps = toml["dependencies"].as_table().expect("no [dependencies]");
    let mut map = BTreeMap::new();
    for (name, val) in deps {
        if HIDDEN_DEPS.contains(&name.as_str()) {
            continue;
        }
        let version = match val {
            toml::Value::String(v) => v.clone(),
            toml::Value::Table(t) => {
                if let Some(v) = t.get("version") {
                    v.as_str().unwrap_or("?").to_string()
                } else if t.contains_key("git") {
                    "git".to_string()
                } else {
                    "?".to_string()
                }
            }
            _ => "?".to_string(),
        };
        map.insert(name.clone(), version);
    }
    serde_json::to_string(&map).expect("failed to serialize deps")
}

fn main() {
    let stable_toml: toml::Value =
        toml::from_str(include_str!("../app/Cargo.toml")).expect("failed to parse app/Cargo.toml");
    let next_toml: toml::Value = toml::from_str(include_str!("../app-next/Cargo.toml"))
        .expect("failed to parse app-next/Cargo.toml");

    let yew_version = stable_toml["dependencies"]["yew"]["version"]
        .as_str()
        .expect("yew version not found in app/Cargo.toml");

    println!("cargo:rustc-env=APP_YEW_VERSION={yew_version}");
    println!(
        "cargo:rustc-env=APP_DEPS_STABLE={}",
        extract_deps(&stable_toml)
    );
    println!(
        "cargo:rustc-env=APP_DEPS_NEXT={}",
        extract_deps(&next_toml)
    );
    println!("cargo::rerun-if-changed=../app/Cargo.toml");
    println!("cargo::rerun-if-changed=../app-next/Cargo.toml");
}
