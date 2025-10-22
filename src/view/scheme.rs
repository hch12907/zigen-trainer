use std::cell::RefCell;
use std::rc::Rc;

use crate::scheme::{LoadedScheme, SchemeOptions};
use crate::user_state::UserState;
use crate::view::card::Card;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use rand::seq::SliceRandom;

#[derive(PartialEq, Clone, Props)]
pub struct SchemeProps {
    scheme_id: String,
    scheme: LoadedScheme,
    options: SchemeOptions,
    on_scheme_completed: EventHandler<()>,
}

#[component]
pub fn Scheme(props: SchemeProps) -> Element {
    let user_state = use_hook(|| {
        let mut state = UserState::read_from_local_storage();
        state.try_initialize_scheme(&props.scheme_id, &props.scheme, props.options);
        tracing::info!("initialized scheme! {}", &props.scheme_id);

        Rc::new(RefCell::new(state))
    });

    let mut zigens = use_signal(|| user_state.borrow().current_progress().get_card().clone());

    let progress = {
        let user_state = user_state.clone();
        use_memo(move || {
            zigens.read();

            let user_state = &*user_state.borrow();
            let completed = user_state.current_progress().reviewed_cards();
            let total = user_state.current_progress().total_cards();

            (completed as f64 / total as f64 * 100.0, completed, total)
        })
    };

    use_effect(|| {
        document::eval(r#"
            document.getElementsByClassName("trainer-tips")[0].classList.add("tips-hidden")
        "#);
    });

    rsx! {
        nav {
            class: "trainer-nav",

            p {
                "进度： {progress().0:.1}% （{progress().1} / {progress().2}）"
            }
        }

        Card {
            zigens: zigens,
            on_card_completed: move |rating| {
                tracing::debug!("completed card! {rating:?}");

                user_state.borrow_mut().current_progress_mut().rate_card(rating);
                user_state.borrow().write_to_local_storage();

                let mut new_card = user_state.borrow().current_progress().get_card().clone();
                
                // 将同个聚类内的归并字根集的顺序打乱，避免发生“首尾记忆”效应（即：只记得前后的字根，中间的易忘）。
                new_card.zigen.as_raw_parts_mut().0.shuffle(&mut rand::rng());
                zigens.set(new_card);
            },
        }

        div {
            class: "trainer-tips",

            p {
                "敲击空格以显示答案"
            }
        }
    }
}
