#![allow(non_snake_case)]
use dioxus_router::prelude::*;
use dioxus::prelude::*;
use ate_model::prelude::*;

use crate::pages::home::view as Home;

pub fn app(cx: Scope) -> Element {
    render!{
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/entries/:id")]
    Entry { id: Uuid },
}

#[inline_props]
fn Entry(cx: Scope, id: Uuid) -> Element {
    render! {
        Link { to: Route::Home {}, "Go to home" }
        "ATE Entry: {id}"
    }
}
