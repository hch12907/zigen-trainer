mod category;
mod category_tree;

use dioxus::prelude::*;

use crate::scheme::Scheme;
use crate::user_state::UserState;
use category::Category;
use category_tree::{CategoryNode, CategoryTree};

#[derive(PartialEq, Clone, Props)]
pub struct SchemeSelectorProps {
    schemes: ReadSignal<Vec<Scheme>>,
    selected_scheme: Signal<String>,
    user_state: ReadSignal<UserState>,
    /// 参数：方案ID、是否跳过设置界面
    on_scheme_selected: EventHandler<(String, bool)>,
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

    let scheme_in_training = {
        let user_state = props.user_state.read();
        let id = user_state.current_scheme();
        
        props.schemes.read().iter().any(|scheme| scheme.id == id).then(|| id.to_owned())
    };

    rsx! {
        if let Some(scheme_id) = scheme_in_training.clone() {
            button {
                class: "selector-confirm-button",
                style: "margin: 2em",
                onclick: move |_| {
                    (props.on_scheme_selected)((scheme_id.clone(), true))
                },
                "继续上次练习"
            }
        }

        p {
            class: "trainer-scheme-selector-description",

            if scheme_in_training.is_none() {
                "选择想要学习的方案后，点击下一步"
            } else {
                "……或在选择想要学习的方案后，点击下一步"
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
            on_scheme_selected: move |scheme_id| (props.on_scheme_selected)((scheme_id, false)),
        }
    }
}
