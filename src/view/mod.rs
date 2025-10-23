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
static GITHUB_LIGHT: Asset = asset!(
    "/assets/github-light.png"
);

#[used]
static GITHUB_DARK: Asset = asset!(
    "/assets/github-dark.png"
);

#[component]
pub fn Trainer() -> Element {
    let mut scheme: Signal<Option<Scheme>> = use_signal(|| None);
    let mut options: Signal<SchemeOptions> = use_signal(|| SchemeOptions::default());

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
                a {
                    onclick: move |_| scheme.set(None),
                    "首页"
                }

                if scheme.read().is_some() {
                    a {
                        onclick: move |_| {
                            if to_confirm_reset() {
                                let mut user_state = UserState::read_from_local_storage();
                                user_state.reset_current_progress();
                                user_state.write_to_local_storage();

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

                a {
                    class: "nav-right",
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

        div {
            class: "trainer-root",

            if scheme.read_unchecked().is_none() {
                Welcome {
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
