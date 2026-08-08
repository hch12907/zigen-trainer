use dioxus::prelude::*;

const KEYBOARD_ROWS: &[&[&str]] = &[
    &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "+", "Backspace"],
    &["Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]", "\\"],
    &["Caps Lock", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "Enter"],
    &["Shift", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "Shift"],
    &["Ctrl", "Win", "Alt", "Space", "Alt", "Win", "Ctrl"],
];

#[derive(PartialEq, Props, Clone)]
pub struct KeyboardProps {
    highlighted_keys: Vec<String>,
}

/// QWERTY 键盘组件
#[component]
pub fn QwertyKeyboard(props: KeyboardProps) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 4px;",
            for row in KEYBOARD_ROWS {
                div {
                    style: "display: flex; gap: 4px; width: 100%;",
                    for key in *row {
                        {
                            let is_highlighted = props.highlighted_keys.iter().any(|k| k == key);

                            // 根据按键类型调整宽度和样式
                            let base_style = if is_highlighted {
                                "padding: 10px 12px; border: 1px solid #aaa; border-radius: 6px; background-color: #f9e74a; font-weight: bold; text-align: center;"
                            } else {
                                "padding: 10px 12px; border: 1px solid #ccc; border-radius: 6px; background-color: #f0f0f0; text-align: center;"
                            };

                            let key_style = match *key {
                                "Backspace" | "\\" | "Tab" | "Space" | "Shift" | "Enter" => "flex: 1;",
                                "Ctrl" | "Alt" | "Win" => "flex: 0.1;",
                                _ => "min-width: 40px;",
                            };

                            rsx! {
                                button {
                                    style: "{base_style} {key_style}",
                                    "{key}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
