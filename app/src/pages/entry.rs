use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app::Route;
use ate_model::prelude::*;

#[inline_props]
pub fn view(cx: Scope, id: Uuid) -> Element {
    render! {
        Link { to: Route::Home {}, "Go to home" }
        "ATE Entry: {id}"
    }
}