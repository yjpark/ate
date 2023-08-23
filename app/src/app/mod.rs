use dioxus::prelude::*;
use dioxus_router::prelude::*;

mod route;
pub mod model;

pub use route::Route;

pub fn app(cx: Scope) -> Element {
    render!{
        Router::<Route> {}
    }
}