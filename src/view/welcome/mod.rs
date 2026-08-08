mod scheme_selector;
mod settings;

use crate::scheme::{Scheme, SchemeOptions};
use crate::user_state::UserState;
use crate::view::welcome::scheme_selector::SchemeSelector;
use crate::view::welcome::settings::Settings;
use dioxus::prelude::*;
use gloo_net::http::Request;

const SCHEME_SELECTOR_CSS: Asset = asset!("/assets/scheme_selector.css");
const SCHEME_SETTINGS_CSS: Asset = asset!("/assets/scheme_settings.css");

#[derive(PartialEq, Clone, Props)]
pub struct WelcomeProps {
    user_state: Signal<UserState>,
    on_scheme_selected: EventHandler<(Scheme, SchemeOptions)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WelcomeState {
    ChooseScheme,
    Settings,
}

#[component]
pub fn Welcome(mut props: WelcomeProps) -> Element {
    let mut selected_scheme = use_signal(|| String::new());
    let mut state = use_signal(|| WelcomeState::ChooseScheme);

    let schemes_loader = {
        use_resource(move || {
            async move {
                let schemes = Request::get("./assets/trainer/schemes.json")
                    .send()
                    .await
                    .map_err(|err| err.to_string())?
                    .json::<Vec<Scheme>>()
                    .await
                    .map_err(|err| err.to_string());

                // let user_state = props.user_state.read();
                // let user_scheme = user_state.current_scheme();
                //
                // // 加载后，如果用户未曾进行过字根练习，默认选择第一个选项，
                // // 否则选择用户上一次练习过的方案
                // if let Ok(ref schemes) = schemes {
                //     if !user_scheme.is_empty() && schemes.iter().any(|scheme| scheme.id == user_scheme) {
                //         selected_scheme.set(user_scheme.to_owned());
                //     } else if let Some(first) = schemes.first() {
                //         selected_scheme.set(first.id.clone());
                //     }
                // }

                schemes
            }
        })
    };

    let schemes = use_memo(move || {
        let loaded_schemes = schemes_loader.read().clone();

        loaded_schemes
            .and_then(|loaded| loaded.ok())
            .unwrap_or(Vec::new())
    });

    use_effect(move || {
        // 随state更新而更新
        state.read();
        document::eval(
            r#"
            document.getElementById("welcome_top").scrollIntoView({ behavior: "smooth" });
        "#,
        );
    });

    rsx! {
        document::Link { rel: "stylesheet", href: SCHEME_SELECTOR_CSS }
        document::Link { rel: "stylesheet", href: SCHEME_SETTINGS_CSS }

        div {
            class: "trainer-welcome",

            h1 {
                id: "welcome_top",
                "慧眼识字根·字根练习器"
            }

            h2 {
                "by hch12907"
            }

            match *schemes_loader.read_unchecked() {
                // 加载成功
                Some(Ok(_)) => {
                    match state() {
                        WelcomeState::ChooseScheme => rsx! {
                            SchemeSelector {
                                schemes: schemes,
                                selected_scheme,
                                user_state: props.user_state,
                                on_scheme_selected: move |(selected_scheme_id, skip_setting): (_, bool)| {
                                    // confirm_reset.set(false);
                                    if !skip_setting {
                                        selected_scheme.set(selected_scheme_id);
                                        state.set(WelcomeState::Settings);
                                    } else {
                                        let scheme = schemes
                                            .read()
                                            .iter()
                                            .find(|scheme| scheme.id == selected_scheme_id)
                                            .cloned()
                                            .unwrap();
                                        (props.on_scheme_selected)((scheme, SchemeOptions::default()))
                                    }
                                }
                            }
                        },

                        WelcomeState::Settings => rsx! {
                            Settings {
                                selected_scheme,
                                user_state: props.user_state,
                                on_back: move || state.set(WelcomeState::ChooseScheme),
                                on_confirm: move |(opts, reset)| {
                                    let scheme = schemes
                                        .read()
                                        .iter()
                                        .find(|scheme| scheme.id == *selected_scheme.read())
                                        .cloned()
                                        .unwrap();
                                    if reset {
                                        props.user_state.write().reset_progress(&scheme.id);
                                    }
                                    (props.on_scheme_selected)((scheme, opts))
                                },
                            }
                        }
                    }
                }

                // 加载失败
                Some(Err(ref e)) => rsx! {
                    p {
                        "数据加载出错！错误信息：{e}"
                    }
                },

                // 尚未加载完成
                _ => rsx! {
                    p {
                        "数据加载中……"
                    }
                }
            }
        }
    }
}
