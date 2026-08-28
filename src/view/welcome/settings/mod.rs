mod setting_option;

use dioxus::prelude::*;

use crate::scheme::{CombineMode, LearnMode, SchemeOptions};
use crate::user_state::UserState;
use setting_option::{BooleanSetting, DropdownSetting, TextboxSetting};

#[derive(Clone, Debug, PartialEq, Props)]
pub struct SettingsProp {
    selected_scheme: ReadSignal<String>,
    user_state: ReadSignal<UserState>,
    on_back: EventHandler<()>,
    /// 参数：方案设置、是否重置
    on_confirm: EventHandler<(SchemeOptions, bool)>,
}

#[component]
pub fn Settings(props: SettingsProp) -> Element {
    let has_existing_session = use_memo(move || {
        let user_state = props.user_state.read();
        let selected_scheme = props.selected_scheme.read();
        user_state.has_progress(&selected_scheme)
    });

    let shuffle = use_signal(|| true);
    let combined_training = use_signal(|| false);
    let prioritize_trad = use_signal(|| false);
    let learn_mode_str = use_signal(String::new);
    let learn_mode = use_memo(move || match learn_mode_str.read().as_str() {
        "adept" => LearnMode::Adept,
        "rapid" => LearnMode::Rapid,
        "novice" | _ => LearnMode::Novice,
    });
    let combined_mode_str = use_signal(String::new);
    let combine_mode = use_memo(move || match combined_mode_str.read().as_str() {
        "group" => CombineMode::Group,
        "none" => CombineMode::None,
        "cluster" | _ => CombineMode::Cluster,
    });
    let limit_keys = use_signal(String::new);
    let v2_sched = use_signal(|| false);

    let mut confirm_reset = use_signal(|| false);
    let mut show_advanced = use_signal(|| false);

    rsx! {

        div {
            class: "scheme-settings",

            div {
                class: "scheme-settings-header",
                id: "settings_header_top",

                h1 {
                    "本轮练习设置"
                }

                p {
                    "这里可以根据个人需求与学习方式，自定义字根的调度算法（即：字根出现的先后顺序）。"
                }
            }

            div {
                class: "scheme-settings-body",

                BooleanSetting {
                    name: "乱序",
                    description: "随机安排字根，而非按字母顺序出现。",
                    value: shuffle,
                }

                BooleanSetting {
                    name: "简繁混练",
                    description: "无论用户是否熟练简体字根，繁体字根都会被安排进入练习队列。",
                    value: combined_training,
                }

                BooleanSetting {
                    name: "繁体优先",
                    description: "优先练习繁体字根。在用户熟练繁体字根后，才会安排简体字根。（简繁混练开启时，本设置无效）",
                    value: prioritize_trad,
                }

                DropdownSetting {
                    name: "学习模式",
                    description: "复习与极速模式都会关闭提示，并且减少每轮练习所需的时间。用于巩固字根记忆。",
                    options: &[
                        ("novice", "普通模式"),
                        ("adept", "复习模式"),
                        ("rapid", "极速模式"),
                    ],
                    value: learn_mode_str,
                }

                // 高级设置
                div {
                    class: "scheme-settings-advanced-section-container",

                    // 高级设置的开关
                    div {
                        class: "scheme-settings-advanced-section-toggle",
                        class: if show_advanced() { "expanded" },
                        tabindex: 0,
                        role: "button",
                        onclick: move |_| {
                            let opened = !show_advanced();
                            show_advanced.set(opened);
                        },

                        span { "高级设置" }
                        span {
                            class: "arrow",
                            // 一个小箭头
                            svg {
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: 2.5,
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                polyline {
                                    points: "6 9 12 15 18 9"
                                }
                            }
                        }
                    }

                    div {
                        class: "scheme-settings-advanced-section",
                        class: if show_advanced() { "expanded" },

                        DropdownSetting {
                            name: "卡片合并模式",
                            description: "调整练习器处理字根卡片的方式。",
                            options: &[
                                ("cluster", "同聚类合并（适合新手）"),
                                ("group", "同归并合并"),
                                ("none", "无合并"),
                            ],
                            value: combined_mode_str,
                        }

                        TextboxSetting {
                            name: "仅训练键面",
                            description: "只练习编码开头为规定键面的字根。留空以训练所有字根。",
                            placeholder: "例：ABCDE",
                            value: limit_keys,
                        }

                        BooleanSetting {
                            name: "使用新型调度器（BETA）",
                            description: "（开发中。）",
                            value: v2_sched,
                        }
                    }
                }

                div {
                    class: "scheme-settings-button",
                    button {
                        class: "selector-confirm-button",
                        onclick: move |_| (props.on_back)(()),
                        "上一步"
                    }

                    if confirm_reset() {
                        p {
                            class: "scheme-settings-reset-label",
                            "该方案已存在学习进程，是否重置？"
                        }
                    }

                    button {
                        class: "selector-confirm-button",
                        onclick: move |_| {
                            let settings = SchemeOptions {
                                shuffle: shuffle(),
                                combined_training: combined_training(),
                                prioritize_trad: prioritize_trad(),
                                learn_mode: learn_mode(),
                                combine_mode: combine_mode(),
                                limit_keys: if !limit_keys.read().is_empty() {
                                    Some(limit_keys.read().chars().map(|c| c.to_ascii_uppercase()).collect())
                                } else {
                                    None
                                },
                                v2_sched: v2_sched()
                            };

                            if !has_existing_session() {
                                (props.on_confirm)((settings, false));
                            } else {
                                if confirm_reset() {
                                    (props.on_confirm)((settings, true));
                                } else {
                                    confirm_reset.set(true);
                                }
                            }
                        },

                        if confirm_reset() {
                            "开始练习（确认）"
                        } else {
                            "开始练习"
                        }
                    }
                }
            }
        }
    }
}
