use std::cell::RefCell;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
// use dioxus_sdk::utils::timing::use_debounce;

use crate::scheduler::{Rating, ZigenCard};

#[derive(PartialEq, Clone, Props)]
pub struct CardProps {
    zigens: ReadSignal<ZigenCard>,
    on_card_completed: EventHandler<Rating>,
}

async fn handle_key_event(
    input_boxes: &mut Memo<Vec<Vec<char>>>,
    event: Event<KeyboardData>,
    mut asked_hint: Memo<bool>,
    mut is_wrong: Signal<bool>,
    confusable: bool,
    expected_answer: &String,
    start_time: Rc<RefCell<DateTime<Utc>>>,
    on_card_completed: EventHandler<Rating>,
) {
    event.stop_propagation();

    if event.is_composing() {
        return;
    }

    match event.key() {
        Key::Character(c) => {
            let c = if c.len() == 1 {
                c.chars().nth(0).unwrap()
            } else {
                return;
            };

            if c == ' ' {
                asked_hint.set(true);
            } else {
                receive_input(input_boxes, c);
            }
        }

        Key::Backspace => {
            remove_input(input_boxes);
        }

        _ => (),
    };

    let filled_up = input_boxes
        .read()
        .iter()
        .enumerate()
        .filter_map(|(i, box_group)| box_group.iter().position(|&c| c == ' ').map(|pos| (i, pos)))
        .next()
        .is_none(); // 当 pos 为 None，证明已无空白输入格。

    if filled_up {
        let user_answer = input_boxes.read().iter().flatten().collect::<String>();

        // 根据用户作答耗时判断该字根的难度，以秒为单位。
        let easy_time = if !confusable {
            2.0 + (expected_answer.len() as f64) * 0.35
        } else {
            2.0 + (expected_answer.len() as f64) * 0.25
        };

        let time_diff = (Utc::now() - *start_time.borrow()).as_seconds_f64();

        clear_input(input_boxes);
        *start_time.borrow_mut() = Utc::now();

        if user_answer == expected_answer.as_str() {
            if !asked_hint() {
                if time_diff <= easy_time {
                    on_card_completed.call(Rating::Easy)
                } else if time_diff <= easy_time + 2.0 {
                    on_card_completed.call(Rating::Good)
                } else {
                    on_card_completed.call(Rating::Hard)
                }
            } else {
                asked_hint.set(false);
                is_wrong.set(false);
                on_card_completed.call(Rating::Again)
            }
        } else {
            asked_hint.set(true);
            is_wrong.set(true);

            // use_debounce(Duration::from_secs(2), move |_| {
            //     is_wrong.set(false);
            // })
            // .action(());

            clear_input(input_boxes);
        }
    }
}

fn receive_input(input_boxes: &mut Memo<Vec<Vec<char>>>, input: char) {
    let pos = input_boxes
        .read()
        .iter()
        .enumerate()
        .filter_map(|(i, box_group)| box_group.iter().position(|&c| c == ' ').map(|pos| (i, pos)))
        .next();

    if let Some((i, j)) = pos {
        input_boxes.write()[i][j] = input.to_ascii_lowercase();
    }
}

fn remove_input(input_boxes: &mut Memo<Vec<Vec<char>>>) {
    let pos = input_boxes
        .read()
        .iter()
        .enumerate()
        .rev()
        .filter_map(|(i, box_group)| {
            box_group
                .iter()
                .rposition(|&c| c != ' ')
                .map(|pos| (i, pos))
        })
        .next();

    if let Some((i, j)) = pos {
        input_boxes.write()[i][j] = ' ';
    }
}

fn clear_input(input_boxes: &mut Memo<Vec<Vec<char>>>) {
    input_boxes
        .write()
        .iter_mut()
        .for_each(|boxes| boxes.iter_mut().for_each(|x| *x = ' '));
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let start_time = use_hook(|| Rc::new(RefCell::new(Utc::now())));

    let asked_hint = use_memo(move || (props.zigens)().is_new_card());
    let is_wrong = use_signal(|| false);

    let zigens = (props.zigens)();

    let mut input_boxes = use_memo(move || {
        let zigens = (props.zigens)();
        let (zigen_groups, _) = zigens.zigen.as_raw_parts();

        let mut boxes = Vec::with_capacity(zigen_groups.len());
        for group in zigen_groups.iter() {
            boxes.push(vec![' '; group.code.chars().count()]);
        }

        boxes
    });

    let expected_answer = use_memo(move || {
        let zigens = (props.zigens)();
        let (zigen_groups, _) = zigens.zigen.as_raw_parts();

        zigen_groups
            .iter()
            .map(|group| group.code.as_str())
            .fold(String::new(), |acc, x| acc + x)
            .to_ascii_lowercase()
    });

    let (zigen_groups, description) = zigens.zigen.as_raw_parts();
    let confusable = matches!(zigens.zigen, crate::scheme::SchemeZigen::Confusable(_));

    let start_time0 = start_time.clone();
    let start_time1 = start_time.clone();

    rsx! {
        div {
            class: "trainer-zigen-card",
            class: if is_wrong() { "flash-red" },

            tabindex: 0,
            onclick: move |_event| {},
            onkeydown: move |event| {
                let start_time0 = start_time0.clone();
                async move {
                    handle_key_event(
                        &mut input_boxes,
                        event,
                        asked_hint,
                        is_wrong,
                        confusable,
                        &*expected_answer.read(),
                        Rc::clone(&start_time0),
                        props.on_card_completed,
                    ).await
                }
            },

            if confusable {
                div {
                    class: "trainer-zigen-confusable",

                    "易混淆字根练习"
                }
            }

            for (i, group) in zigen_groups.iter().enumerate() {
                div {
                    class: "trainer-zigen-group",

                    div {
                        class: "trainer-zigen-display",

                        for zigen in &group.zigens {
                            p {
                                "{zigen.0}"
                            }
                        }
                    }

                    div {
                        class: "trainer-zigen-inputs",

                        for (j, _) in group.code.chars().enumerate() {
                            {
                                let start_time2 = start_time1.clone();

                                rsx! {
                                        div {
                                        class: "trainer-zigen-input",
                                        autofocus: true,
                                        tabindex: 0,
                                        onclick: move |_event| {},
                                        onkeydown: move |event| {
                                            let start_time2 = start_time2.clone();
                                            async move {
                                                handle_key_event(
                                                    &mut input_boxes,
                                                    event,
                                                    asked_hint,
                                                    is_wrong,
                                                    confusable,
                                                    &*expected_answer.read(),
                                                    Rc::clone(&start_time2),
                                                    props.on_card_completed,
                                                ).await
                                            }
                                        },
                                        "{input_boxes.read()[i][j]}"
                                    }
                                }
                            }
                        }
                    }

                    if *asked_hint.read() {
                        div {
                            class: "trainer-zigen-group-answer",
                            p {
                                "答案：{group.code}"
                            }
                        }
                        div {
                            class: "trainer-zigen-group-description",
                            div {
                                dangerous_inner_html: "{group.description}"
                            }
                        }
                    }
                }
            }

            if *asked_hint.read() {
                div {
                    class: "trainer-zigen-description",
                    dangerous_inner_html: "{description}",
                }
            }
        }
    }
}
