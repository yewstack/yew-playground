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

#[macro_export]
macro_rules! rc_type {
    ($ident:ident => $ty:ty) => {
        #[derive(Clone)]
        struct $ident(::std::rc::Rc<$ty>);

        impl ::std::ops::Deref for $ident {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl PartialEq for $ident {
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.0, &other.0)
            }
        }
    };
}
