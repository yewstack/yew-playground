fn main() {
    let toml: toml::Value =
        toml::from_str(include_str!("../app/Cargo.toml")).expect("failed to parse app/Cargo.toml");

    let yew_version = toml["dependencies"]["yew"]["version"]
        .as_str()
        .expect("yew version not found in app/Cargo.toml");

    println!("cargo:rustc-env=APP_YEW_VERSION={yew_version}");
    println!("cargo::rerun-if-changed=../app/Cargo.toml");
}
