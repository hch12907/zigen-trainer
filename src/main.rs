mod component;
mod scheduler;
mod scheduler_fsrs;
mod scheme;
mod user_state;
mod view;

use dioxus::prelude::*;

use crate::view::Trainer;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Trainer {}
    }
}
