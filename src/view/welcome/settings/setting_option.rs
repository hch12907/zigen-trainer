use dioxus::prelude::*;

#[component]
fn Setting(
    name: &'static str,
    description: &'static str,
    children: Element
) -> Element {
    rsx! {
        div {
            class: "scheme-setting-row",

            div {
                class: "scheme-setting-info",

                h3 {
                    "{name}"
                }

                p {
                    "{description}"
                }
            }

            {children}
        }
    }
}

#[component]
pub fn BooleanSetting(
    name: &'static str,
    description: &'static str,
    mut value: Signal<bool>,
) -> Element {
    rsx! {
        Setting {
            name,
            description,

            label {
                class: "scheme-setting-toggle",

                input {
                    r#type: "checkbox",
                    checked: value(),
                    onclick: move |_| {
                        let new_value = !value();
                        value.set(new_value);
                    }
                }
                span {
                    class: "scheme-setting-toggle-slider"
                }
            }
        }
    }
}

#[component]
pub fn DropdownSetting(
    name: &'static str,
    description: &'static str,
    options: &'static [(&'static str, &'static str)],
    mut value: Signal<String>,
) -> Element {
    rsx! {
        Setting {
            name,
            description,

            select {
                class: "scheme-setting-dropdown",

                onchange: move |event| {
                    value.set(event.value());
                },

                for (value, label) in options.iter() {
                    option {
                        value,
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn TextboxSetting(
    name: &'static str,
    description: &'static str,
    placeholder: &'static str,
    mut value: Signal<String>,
) -> Element {
    rsx! {
        Setting {
            name,
            description,

            input {
                class: "scheme-setting-textbox",
                r#type: "text",
                placeholder,
                oninput: move |event| { value.set(event.value()) },
            }
        }
    }
}
