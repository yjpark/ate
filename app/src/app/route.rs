#![allow(non_snake_case)]
use dioxus_router::prelude::*;
use dioxus::prelude::*;
use ate_model::prelude::*;

use crate::pages::home::view as Home;
use crate::pages::history::view as History;
use crate::pages::entry::view as Entry;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/history")]
    History {},
    #[route("/entries/:id")]
    Entry { id: Uuid },
}