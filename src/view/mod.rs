mod card;
mod scheme;
mod welcome;

use dioxus_logger::tracing;
use gloo_net::http::Request;
use web_sys::FontFace;
use web_sys::wasm_bindgen::prelude::Closure;
pub use welcome::*;

use dioxus::prelude::*;

use crate::scheme::{LoadedScheme, Scheme, SchemeOptions, ZigenConfusableUnpopulated};
use crate::user_state::UserState;

#[used]
static SCHEMES: Asset = asset!(
    "/assets/trainer/",
    AssetOptions::folder().with_hash_suffix(false)
);

#[used]
static GITHUB_LIGHT: Asset = asset!("/assets/github-light.png");

#[used]
static GITHUB_DARK: Asset = asset!("/assets/github-dark.png");

#[used]
static TUTORIAL_PAGE: Asset = asset!(
    "/assets/tutorial.html",
    AssetOptions::builder().with_hash_suffix(false)
);

#[component]
pub fn Trainer() -> Element {
    let mut scheme: Signal<Option<Scheme>> = use_signal(|| None);
    let mut options: Signal<SchemeOptions> = use_signal(|| SchemeOptions::default());
    let mut user_state: Signal<UserState> = use_signal(|| UserState::read_from_local_storage());

    let loaded_scheme = use_resource(move || async move {
        if let Some(scheme) = &*scheme.read() {
            if !scheme.zigen_font.is_empty() {
                let zigen_font = FontFace::new_with_str(
                    "zigen-font",
                    &format!("url(./assets/trainer/{})", &scheme.zigen_font),
                );

                if let Ok(zigen_font) = zigen_font {
                    if let Ok(promise) = zigen_font.load() {
                        let closure = Closure::new(|font| {
                            web_sys::window()
                                .and_then(|window| window.document())
                                .map(|document| document.fonts())
                                .map(|fonts| fonts.add(&FontFace::from(font)));
                        });

                        let _ = promise.then(&closure);

                        // 把闭包交给JS的垃圾回收器处理，不然在promise执行前它早就已经被
                        // Rust自己的drop代码处理了
                        closure.forget();
                    }
                } else {
                    tracing::warn!("unable to load zigen font from {}", scheme.id);
                }
            }

            Request::get(&(String::from("./assets/trainer/") + &scheme.zigen_url))
                .send()
                .await
                .map_err(|err| err.to_string())?
                .json::<LoadedScheme<ZigenConfusableUnpopulated>>()
                .await
                .map_err(|err| err.to_string())
                .map(|loaded| (scheme.id.clone(), loaded))
        } else {
            Err(String::new())
        }
    });

    let mut to_confirm_reset = use_signal(|| false);

    rsx! {
        div {
            nav {
                class: "root-nav",

                a {
                    onclick: move |_| scheme.set(None),
                    "首页"
                }

                if scheme.read().is_some() {
                    a {
                        onclick: move |_| {
                            if to_confirm_reset() {
                                user_state.write().reset_current_progress();
                                user_state.read().write_to_local_storage();

                                scheme.set(None);
                                to_confirm_reset.set(false);
                            } else {
                                to_confirm_reset.set(true);
                            }
                        },

                        if to_confirm_reset() {
                            "重启？（将会重置所有进度！）"
                        } else {
                            "重启"
                        }
                    }
                }

                div {
                    class: "nav-right",
                    a {
                        onclick: move |_| {
                            document::eval(r#"document.location.href = "./assets/tutorial.html""#);
                        },
                        "使用教程"
                    }

                    a {
                        onclick: move |_| {
                            document::eval(r#"document.getElementById("import-file-button").click();"#);
                        },
                        "导入进度"
                    }

                    input {
                        style: "display:none",
                        r#type: "file",
                        id: "import-file-button",
                        accept: "application/json",
                        multiple: false,
                        onchange: move |event| async move {
                            let files = event.files();
                            if let Some(file) = files.get(0) {
                                let content = file.read_string().await;

                                match content {
                                    Ok(content) => {
                                        if user_state.write().load_from_backup(content).is_err() {
                                            document::eval(r#"
                                                let message = await dioxus.recv();
                                                alert("无法解析备份文件！原因：" + message);
                                            "#);    
                                        }
                                        user_state.read().write_to_local_storage();
                                    }
                                    Err(_) => {
                                        document::eval(r#"alert("无法加载文件！")"#);
                                    }
                                }
                            }
                        }
                    }

                    a {
                        onclick: move |_| {
                            use_effect(|| {
                                document::eval(r#"
                                    const time_now = new Date().toISOString().replaceAll(':', '-');
                                    const content = localStorage.progresses || {};
                                    const blob = new Blob([content], { type: "application/json" });
                                    const url = URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.download = `慧根进度备份${time_now}.json`;
                                    a.click();
                                    URL.revokeObjectURL(url);
                                "#);
                            });
                        },
                        "导出进度"
                    }

                    a {
                        href: "https://github.com/hch12907/zigen-trainer",

                        picture {
                            source {
                                "srcset": GITHUB_LIGHT,
                                "media": "(prefers-color-scheme: light)",
                            }
                            source {
                                "srcset": GITHUB_DARK,
                                "media": "(prefers-color-scheme: dark)",
                            }
                            img {
                                src: GITHUB_LIGHT,
                            }
                        }
                    }
                }
            }
        }

        div {
            class: "trainer-root",

            if scheme.read_unchecked().is_none() {
                Welcome {
                    user_state,
                    on_scheme_selected: move |(selected, opts)| {
                        scheme.set(Some(selected));
                        options.set(opts);
                    }
                }
            }

            match &*loaded_scheme.read_unchecked() {
                None => rsx! {
                    p {
                        "数据加载中……"
                    }
                },

                Some(Ok((scheme_id, scheme))) => rsx! {
                    scheme::Scheme {
                        scheme_id: scheme_id,
                        scheme: scheme.clone(),
                        options: options(),
                        user_state,
                        on_scheme_completed: |()| {},
                    }
                },

                Some(Err(e)) if e.is_empty() => rsx! {},

                Some(Err(e)) => rsx! {
                    p {
                        "数据加载失败！错误信息：{e}"
                    }
                }
            }
        }
    }
}
