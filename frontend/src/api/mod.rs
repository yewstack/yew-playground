pub mod run;
pub use run::run;


pub const BACKEND_URL: &str = option_env!("BACKEND_URL").unwrap_or({
    #[cfg(debug_assertions)]
    const DEFAULT: &str = "http://localhost:3000";

    #[cfg(not(debug_assertions))]
    const DEFAULT: &str = "https://api.play.yew.rs";

    DEFAULT
});
