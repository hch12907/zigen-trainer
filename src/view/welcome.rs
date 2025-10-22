use crate::scheme::{Scheme, SchemeOptions};
use dioxus::prelude::*;
use gloo_net::http::Request;

#[derive(PartialEq, Clone, Props)]
pub struct WelcomeProps {
    on_scheme_selected: EventHandler<(Scheme, SchemeOptions)>,
}

#[component]
pub fn Welcome(props: WelcomeProps) -> Element {
    let mut selected_scheme = use_signal(|| String::new());
    let mut shuffle = use_signal(|| false);
    let mut combined_training = use_signal(|| true);
    let mut prioritize_trad = use_signal(|| false);
    let mut adept = use_signal(|| false);

    let schemes = use_resource(move || async move {
        let schemes = Request::get("./assets/trainer/schemes.json")
            .send()
            .await
            .map_err(|err| err.to_string())?
            .json::<Vec<Scheme>>()
            .await
            .map_err(|err| err.to_string());

        // 加载后，默认选择第一个选项
        if let Ok(ref schemes) = schemes {
            if let Some(first) = schemes.first() {
                selected_scheme.set(first.id.clone());
            }
        }

        schemes
    });

    rsx! {
        div {
            class: "trainer-welcome",

            h1 {
                "字根练习器"
            }

            h2 {
                "by hch12907"
            }

            match &*schemes.read_unchecked() {
                // 加载成功
                Some(Ok(read_schemes)) => {
                    rsx!{
                        select {
                            class: "trainer-scheme-selector",
                            id: "trainer-scheme",
                            onchange: move |event| selected_scheme.set(event.value()),

                            for scheme in read_schemes {
                                option {
                                    key: "{scheme.id}",
                                    value: "{scheme.id}",
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
                                "简繁通练"
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
                                };
                                props.on_scheme_selected.call((scheme.unwrap(), options));
                            },

                            "开始练习"
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
