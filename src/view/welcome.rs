use crate::scheme::{CombineMode, Scheme, SchemeOptions};
use crate::user_state::UserState;
use dioxus::prelude::*;
use gloo_net::http::Request;

#[derive(PartialEq, Clone, Props)]
pub struct WelcomeProps {
    on_scheme_selected: EventHandler<(Scheme, SchemeOptions)>,
}

#[component]
pub fn Welcome(props: WelcomeProps) -> Element {
    let user_state = UserState::read_from_local_storage();

    let mut selected_scheme = use_signal(|| String::new());
    let mut shuffle = use_signal(|| false);
    let mut combined_training = use_signal(|| false);
    let mut prioritize_trad = use_signal(|| false);
    let mut adept = use_signal(|| false);
    let mut combine_mode = use_signal(|| CombineMode::Category);
    let mut limit_keys = use_signal(|| String::new());
    let mut confirm_reset = use_signal(|| false);

    let schemes = {
        let user_state = user_state.clone();

        use_resource(move || {
            let user_state = user_state.clone();
            async move {
                let schemes = Request::get("./assets/trainer/schemes.json")
                    .send()
                    .await
                    .map_err(|err| err.to_string())?
                    .json::<Vec<Scheme>>()
                    .await
                    .map_err(|err| err.to_string());

                let user_scheme = user_state.current_scheme();

                // 加载后，如果用户未曾进行过字根练习，默认选择第一个选项，
                // 否则选择用户上一次练习过的方案
                if let Ok(ref schemes) = schemes {
                    if !user_scheme.is_empty() && schemes.iter().any(|scheme| scheme.id == user_scheme) {
                        selected_scheme.set(user_scheme.to_owned());
                    } else if let Some(first) = schemes.first() {
                        selected_scheme.set(first.id.clone());
                    }
                }
                
                schemes
            }
        })
    };

    let show_continue = {
        let selected_scheme = selected_scheme.clone();

        use_memo(move || {
            user_state.has_progress(&selected_scheme())
        })
    };

    let make_button = |name: &'static str, onclicked: Option<Box<dyn Fn() -> bool>>| {
        rsx! {
            button {
                onclick: move |_event| {
                    let name = &*selected_scheme.read();
                    let scheme = (*schemes
                        .read_unchecked())
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|s| s.id == *name)
                        .cloned();
                    let options = SchemeOptions {
                        shuffle: shuffle(),
                        combined_training: combined_training(),
                        prioritize_trad: prioritize_trad(),
                        adept: adept(),
                        combine_mode: combine_mode(),
                        limit_keys: if !limit_keys().is_empty() {
                            Some(limit_keys().chars().map(|c| c.to_ascii_uppercase()).collect())
                        } else {
                            None
                        },
                    };
                    let start_training = {
                        if let Some(onclicked) = &onclicked {
                            onclicked()
                        } else {
                            true
                        }
                    };

                    if start_training {
                        props.on_scheme_selected.call((scheme.unwrap(), options));
                    }
                },

                "{name}"
            }
        }
    };

    rsx! {
        div {
            class: "trainer-welcome",

            h1 {
                "慧眼识根·字根练习器"
            }

            h2 {
                "hch12907 制作"
            }

            match &*schemes.read_unchecked() {
                // 加载成功
                Some(Ok(read_schemes)) => {
                    rsx!{
                        select {
                            class: "trainer-scheme-selector",
                            id: "trainer-scheme",
                            onchange: move |event| {
                                confirm_reset.set(false);
                                selected_scheme.set(event.value())
                            },

                            for scheme in read_schemes {
                                option {
                                    key: "{scheme.id}",
                                    value: "{scheme.id}",
                                    selected: selected_scheme() == scheme.id,
                                    "{scheme.full_name}",
                                }
                            }
                        }

                        div {
                            class: "trainer-scheme-options",

                            input {
                                r#type: "checkbox",
                                id: "shuffle_zigens",
                                checked: shuffle(),
                                onchange: move |_event| {
                                    shuffle.set(!shuffle());
                                }
                            }

                            label {
                                r#for: "shuffle_zigens",
                                "乱序"
                            }

                            input {
                                r#type: "checkbox",
                                id: "combined_training",
                                checked: combined_training(),
                                onchange: move |_event| {
                                    combined_training.set(!combined_training());
                                }
                            }

                            label {
                                r#for: "combined_training",
                                "简繁混练"
                            }

                            input {
                                r#type: "checkbox",
                                id: "prioritize_trad",
                                checked: prioritize_trad(),
                                onchange: move |_event| {
                                    prioritize_trad.set(!prioritize_trad());
                                }
                            }

                            label {
                                r#for: "prioritize_trad",
                                "繁体优先"
                            }

                            input {
                                r#type: "checkbox",
                                id: "adept",
                                checked: adept(),
                                onchange: move |_event| {
                                    adept.set(!adept());
                                }
                            }

                            label {
                                r#for: "adept",
                                "养老模式"
                            }
                        }

                        div {
                            class: "trainer-welcome-buttons",

                            if show_continue() {
                                { make_button("继续练习", None) }
                                { 
                                    let confirm_reset = confirm_reset.clone();
                                    let button_name = if confirm_reset() {
                                        "重置练习（确认？）"
                                    } else {
                                        "重置练习"
                                    };
                                    make_button(button_name, Some(Box::new(move || {
                                        let mut confirm_reset = confirm_reset;

                                        if confirm_reset() {
                                            let mut user_state = UserState::read_from_local_storage();
                                            user_state.reset_progress(&selected_scheme());
                                            user_state.write_to_local_storage();

                                            confirm_reset.set(false);
                                            true
                                        } else {
                                            confirm_reset.set(true);
                                            false
                                        }
                                    })))
                                }
                            } else {
                                { make_button("开始练习", None) }
                            }
                        }

                        details {
                            class: "trainer-scheme-advanced-options",

                            summary {
                                "高级设置"
                            }

                            label {
                                r#for: "trainer-combine-mode",
                                "卡片合并模式："
                            }

                            select {
                                class: "trainer-combine-mode-selector",
                                id: "trainer-combine-mode",
                                onchange: move |event| match event.value().as_str() {
                                    "category" => combine_mode.set(CombineMode::Category),
                                    "group" => combine_mode.set(CombineMode::Group),
                                    "none" => combine_mode.set(CombineMode::None),
                                    _ => (),
                                },

                                option {
                                    value: "category",
                                    selected: combine_mode() == CombineMode::Category,
                                    "同聚类合并（适合新手）"
                                }

                                option {
                                    value: "group",
                                    selected: combine_mode() == CombineMode::Group,
                                    "同归并合并"
                                }

                                option {
                                    value: "none",
                                    selected: combine_mode() == CombineMode::None,
                                    "无合并"
                                }
                            }

                            div { class: "break-flex-row" }

                            label {
                                r#for: "trainer-key-limit",
                                "仅训练键面："
                            }

                            input {
                                r#type: "text",
                                placeholder: "ABCDE（留空以训练所有字根）",
                                oninput: move |event| limit_keys.set(event.value().trim().to_owned()),
                            }
                        }
                    }
                },

                // 加载失败
                Some(Err(e)) => rsx! {
                    p {
                        "数据加载出错！错误信息：{e}"
                    }
                },

                // 尚未加载完成
                None => rsx! {
                    p {
                        "数据加载中……"
                    }
                }
            }
        }
    }
}
