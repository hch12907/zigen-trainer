use std::cell::RefCell;
use std::rc::Rc;

use crate::scheduler::SchedulerCard;
use crate::scheme::{
    LoadedScheme, SchemeOptions, ZigenConfusableUnpopulated,
};
use crate::user_state::UserState;
use crate::view::card::Card;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use rand::seq::SliceRandom;

#[derive(PartialEq, Clone, Props)]
pub struct SchemeProps {
    scheme_id: String,
    scheme: LoadedScheme<ZigenConfusableUnpopulated>,
    options: SchemeOptions,
    on_scheme_completed: EventHandler<()>,
}

#[component]
pub fn Scheme(props: SchemeProps) -> Element {
    let user_state = use_hook(|| {
        let mut state = UserState::read_from_local_storage();
        let res = state.try_initialize_scheme(&props.scheme_id, &props.scheme, props.options);
        tracing::info!("initialized scheme! {}", &props.scheme_id);

        res.map(|_| Rc::new(RefCell::new(state)))
    });

    let mut zigens = use_signal(|| match user_state.clone() {
        Ok(state) => state.borrow_mut().current_progress_mut().get_card(),
        Err(_) => Box::new(SchedulerCard::default()),
    });

    let adept = use_signal(|| match user_state.clone() {
        Ok(state) => state.borrow().current_progress().is_adept(),
        Err(_) => false,
    });

    let progress = {
        let user_state = user_state.clone();
        use_memo(move || {
            zigens.read();

            let (completed, total) = match &user_state {
                Ok(user_state) => {
                    let user_state = &*user_state.borrow();
                    let completed = user_state.current_progress().reviewed_cards();
                    let total = user_state.current_progress().total_cards();
                    (completed, total)
                }

                Err(_) => (0, 0),
            };

            (completed as f64 / total as f64 * 100.0, completed, total)
        })
    };

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

        match user_state {
            Ok(user_state) => rsx! {
                Card {
                    zigens: zigens,
                    adept: adept(),
                    on_card_completed: move |rating| {
                        tracing::debug!("completed card! {rating:?}");

                        user_state.borrow_mut().current_progress_mut().rate_card(rating);
                        user_state.borrow().write_to_local_storage();

                        let mut new_card = user_state.borrow_mut().current_progress_mut().get_card();

                        // 将同个聚类内的归并字根集的顺序打乱，避免发生“首尾记忆”效应（即：只记得前后的字根，中间的易忘）。
                        new_card.zigen_mut().as_raw_parts_mut().0.shuffle(&mut rand::rng());
                        zigens.set(new_card);
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
