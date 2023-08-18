use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::app::Route;
use ate_model::prelude::*;

#[inline_props]
pub fn view(cx: Scope) -> Element {
    render!{
        div { "History" }
        Link { to: Route::Entry { id: Uuid::new_v4() }, "Go to entry" }
    }
}