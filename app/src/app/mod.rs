use dioxus::prelude::*;
use dioxus_router::prelude::*;

pub mod model;
pub mod route;

pub use route::Route;

pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}