#[macro_export]
macro_rules! icon {
    (@import $name:literal) => {{
        include_str!(concat!("../node_modules/@material-design-icons/svg/filled/", $name, ".svg"))
    }};
    ($name:literal) => {{
        $crate::utils::html_to_element(icon!(@import $name), None)
    }};
    ($name:literal, $classes:expr) => {{
        $crate::utils::html_to_element(icon!(@import $name), Some($classes))
    }};
}
