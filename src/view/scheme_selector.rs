use dioxus::prelude::*;

use crate::scheme::Scheme;

#[derive(PartialEq, Clone, Props)]
pub struct SchemeSelectorProps {
    schemes: ReadSignal<Vec<Scheme>>,
    selected_scheme: Signal<String>,
    on_scheme_selected: EventHandler<String>,
}

#[component]
pub fn SchemeSelector(props: SchemeSelectorProps) -> Element {
    rsx! {
        select {
            class: "trainer-scheme-selector",
            id: "trainer-scheme",
            onchange: move |event| {
                props.on_scheme_selected.call(event.value());
            },

            for scheme in props.schemes.iter() {
                option {
                    key: "{scheme.id}",
                    value: "{scheme.id}",
                    selected: *props.selected_scheme.read() == scheme.id,
                    "{scheme.full_name}",
                }
            }
        }
    }
}
