use crate::scheme::{
    LoadedScheme, SchemeOptions, ZigenConfusableUnpopulated,
};
use crate::user_state::UserState;
use crate::view::card::Card;

use dioxus::prelude::*;
use dioxus_logger::tracing;

#[derive(PartialEq, Clone, Props)]
pub struct SchemeProps {
    scheme_id: String,
    scheme: LoadedScheme<ZigenConfusableUnpopulated>,
    options: SchemeOptions,
    user_state: Signal<UserState>,
    on_scheme_completed: EventHandler<()>,
}

#[component]
pub fn Scheme(mut props: SchemeProps) -> Element {
    let res = props.user_state.write().try_initialize_scheme(&props.scheme_id, &props.scheme, props.options);
    assert!(!res.is_err());
    tracing::info!("initialized scheme! {}", &props.scheme_id);

    let zigens = props.user_state.write().current_progress_mut().get_card();
    let adept = props.user_state.read().current_progress().is_adept();

    let progress = use_memo(move || {
        let user_state = props.user_state.read();
        let current_progress = user_state.current_progress();
        let completed = current_progress.reviewed_cards();
        let total = current_progress.total_cards();

        (completed as f64 / total as f64 * 100.0, completed, total)
    });

    use_effect(|| {
        document::eval(
            r#"
            document.getElementsByClassName("trainer-tips")[0].classList.add("tips-hidden")
        "#,
        );
    });

    rsx! {
        nav {
            class: "trainer-nav",

            p {
                "进度： {progress().0:.1}% （{progress().1} / {progress().2}）"
            }
        }

        match res {
            Ok(()) => rsx! {
                Card {
                    zigens: zigens,
                    adept: adept,
                    on_card_completed: move |rating| {
                        tracing::debug!("completed card! {rating:?}");

                        let mut user_state = props.user_state.write();
                        user_state.current_progress_mut().rate_card(rating);

                        // 将同个聚类内的归并字根集的顺序打乱，避免发生“首尾记忆”效应（即：记住了前后的字根，而中间的却忘了）。
                        user_state.current_progress_mut().get_card_mut().shuffle();

                        user_state.write_to_local_storage();
                    },
                }
            },

            Err(msg) => rsx! {
                p {
                    "发生了错误！（{msg}）请刷新本页面。"
                }
            }
        }

        div {
            class: "trainer-tips",

            p {
                "敲击空格以显示答案"
            }
        }
    }
}
