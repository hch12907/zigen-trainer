mod category;
mod category_tree;

use dioxus::prelude::*;

use crate::scheme::Scheme;
use category::Category;
use category_tree::{CategoryNode, CategoryTree};

const SCHEME_SELECTOR_CSS: Asset = asset!("/assets/scheme_selector.css");

#[derive(PartialEq, Clone, Props)]
pub struct SchemeSelectorProps {
    schemes: ReadSignal<Vec<Scheme>>,
    selected_scheme: Signal<String>,
    on_scheme_selected: EventHandler<String>,
}

#[component]
pub fn SchemeSelector(props: SchemeSelectorProps) -> Element {
    let categories = use_memo(move || CategoryTree::new(&*props.schemes.read()));

    let main_categories = use_memo(move || {
        categories
            .read()
            .children()
            .map(|cat| cat.name().to_owned())
            .collect::<Vec<_>>()
    });

    let mut selected_main_category_idx = use_signal(|| 0);

    let selected_main_category = use_memo(move || {
        categories
            .read()
            .children()
            .nth(selected_main_category_idx())
            .unwrap()
            .clone()
    });

    rsx! {
        document::Link { rel: "stylesheet", href: SCHEME_SELECTOR_CSS }

        p {
            class: "trainer-scheme-selector-description",
            "选择想要学习的方案后，点击下一步"
        }

        select {
            style: "display:none",
            class: "trainer-scheme-selector",
            id: "trainer-scheme",
            onchange: move |event| {
                props.on_scheme_selected.call(event.value());
            },

            for scheme in props.schemes.iter() {
                option {
                    key: "{scheme.id}",
                    value: "{scheme.id}",
                    selected: *props.selected_scheme.read() == scheme.id,
                    "{scheme.full_name}",
                }
            }
        }

        div {
            class: "selector-category-tabs",
            for (i, category) in main_categories.iter().enumerate() {
                button {
                    class: "selector-category-tab",
                    class: if selected_main_category_idx() == i { "active" },
                    onclick: move |_| { selected_main_category_idx.set(i) },
                    "{category}"
                }
            }
        }

        Category {
            category: selected_main_category,
            schemes: props.schemes,
            on_scheme_selected: move |scheme_id| (props.on_scheme_selected)(scheme_id),
        }
    }
}
