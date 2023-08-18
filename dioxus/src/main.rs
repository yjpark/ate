use log::LevelFilter;
use ate_dioxus::app::app;

#[cfg(feature = "desktop")]
fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    dioxus_desktop::launch(app);
}

#[cfg(feature = "web")]
fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    dioxus_web::launch(app);
}
