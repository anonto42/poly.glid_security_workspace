mod backend;
mod ui;

use dioxus::prelude::*;
use ui::App;

fn main() {
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("PolyGlid Developer Space")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(900.0, 620.0));
    let config = dioxus::desktop::Config::new()
        .with_window(window)
        .with_menu(None);
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! { config })
        .launch(App);
}
