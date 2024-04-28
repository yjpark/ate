use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::pages::*;

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route {
    #[route("/")]
    Home {},
}