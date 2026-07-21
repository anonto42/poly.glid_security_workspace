use dioxus::prelude::*;

#[component]
pub(crate) fn RailButton(
    icon: &'static str,
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "rail-button active" } else { "rail-button" }, title: label, onclick: move |event| onclick.call(event), span { "{icon}" } small { "{label}" } } }
}

#[component]
pub(crate) fn BottomTabButton(
    label: &'static str,
    count: Option<usize>,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "bottom-tab active" } else { "bottom-tab" }, onclick: move |event| onclick.call(event), "{label}" if let Some(value) = count { span { "{value}" } } } }
}

#[component]
pub(crate) fn SettingsButton(
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "active" } else { "" }, onclick: move |event| onclick.call(event), "{label}" } }
}
