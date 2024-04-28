use std::env;
use std::path::Path;

use dioxus_daisyui::build::generate_classes;
use dioxus_daisyui::prelude::*;

pub struct Units(pub f32);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=classes.rs");
    // let out_dir = env::var_os("OUT_DIR").unwrap();
    let current_dir = env::current_dir()?;

    let classes_path = Path::new(&current_dir).join("classes.html");
    let classes = include!("classes.rs");
    generate_classes(classes_path, classes)?;
    
    dioxus_daisyui::build::tailwindcss(current_dir, "tailwind.input.css", "../assets/css/tailwind.css")
}
