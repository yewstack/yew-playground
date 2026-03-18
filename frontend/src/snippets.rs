pub struct Snippet {
    pub name: &'static str,
    pub code: &'static str,
}

pub const STABLE_SNIPPETS: &[Snippet] = &[
    Snippet {
        name: "Hello World",
        code: include_str!("../../app/snippets/hello_world.rs"),
    },
    Snippet {
        name: "Styled Counter",
        code: include_str!("../../app/snippets/styled_counter.rs"),
    },
];

pub const NEXT_SNIPPETS: &[Snippet] = &[Snippet {
    name: "Hello World",
    code: include_str!("../../app-next/snippets/hello_world.rs"),
}];

pub fn snippets_for(version: &str) -> &'static [Snippet] {
    if version == "next" {
        NEXT_SNIPPETS
    } else {
        STABLE_SNIPPETS
    }
}
