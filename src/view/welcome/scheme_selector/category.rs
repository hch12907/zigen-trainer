use std::collections::BTreeMap;

use crate::scheme::Scheme;

use super::CategoryNode;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct CategoryProps {
    category: ReadSignal<CategoryNode>,
    schemes: ReadSignal<Vec<Scheme>>,
    on_scheme_selected: EventHandler<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SchemeInfo {
    id: String,
    name: String,
    icon: String,
}

#[component]
pub fn Category(props: CategoryProps) -> Element {
    let schemes = use_memo(move || {
        props
            .schemes
            .read()
            .iter()
            .filter(|scheme| {
                scheme
                    .category
                    .first()
                    .map(|cat| cat == props.category.read().name())
                    .unwrap_or_default()
            })
            .map(|scheme| {
                (
                    scheme
                        .category
                        .split_first()
                        .map(|(_, rest)| rest.join(""))
                        .unwrap_or_default(),
                    SchemeInfo {
                        id: scheme.id.to_owned(),
                        name: scheme.full_name.to_owned(),
                        icon: scheme.icon.to_owned(),
                    },
                )
            })
            .fold(
                // 注意: 如果有必要，这里可以改成indexmap的
                BTreeMap::<String, Vec<SchemeInfo>>::new(),
                |mut map, (category, scheme_info)| {
                    map.entry(category).or_default().push(scheme_info);
                    map
                },
            )
    });

    let mut selected_scheme: Signal<String> = use_signal(String::new);
    let selected_scheme_name = use_memo(move || {
        if !selected_scheme.read().is_empty() {
            let schemes = props.schemes.read();
            let scheme = schemes
                .iter()
                .find(|scheme| scheme.id == *selected_scheme.read());
            scheme.unwrap().full_name.clone()
        } else {
            String::new()
        }
    });

    rsx! {
        div {
            class: "selector-category-body",

            for (category, infos) in schemes.read().iter() {
                h3 {
                    class: "selector-subcategory-label",
                    "{category}"
                }

                div {
                    class: "selector-subcategory-container",
                    for info in infos.iter() {
                        div {
                            class: "selector-scheme-card",
                            class: if *selected_scheme.read() == info.id { "active" },
                            onclick: {
                                let id = info.id.clone();
                                move |_| {
                                    selected_scheme.set(id.to_owned());
                                }
                            },

                            div {
                                class: "selector-scheme-card-icon",

                                if info.icon.contains('/') {
                                    img { src: "{info.icon}" }
                                } else {
                                    "{info.icon}"
                                }
                            }

                            div {
                                class: "selector-scheme-card-name",
                                "{info.name}"
                            }
                        }
                    }
                }
            }

            div {
                class: "selector-confirm-section",

                p {
                    class: "selector-selected-scheme-label",

                    if !selected_scheme.read().is_empty() {
                        "已选：{selected_scheme_name}"
                    } else {
                        ""
                    }
                }

                button {
                    class: "selector-confirm-button",
                    disabled: selected_scheme.read().is_empty(),
                    onclick: move |_| {
                        if !selected_scheme.read().is_empty() {
                            (props.on_scheme_selected)(selected_scheme.read().to_owned())
                        }
                    },
                    "下一步"
                }
            }
        }
    }
}
