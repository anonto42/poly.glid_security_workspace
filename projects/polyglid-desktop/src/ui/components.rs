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
pub(crate) fn SidebarOption(label: &'static str, meta: &'static str, active: bool) -> Element {
    rsx! { button { class: if active { "sidebar-option active" } else { "sidebar-option" }, span { "{label}" } small { "{meta}" } } }
}

#[component]
pub(crate) fn EditorTabButton(
    label: &'static str,
    icon: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "editor-tab active" } else { "editor-tab" }, onclick: move |event| onclick.call(event), span { "{icon}" } "{label}" } }
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

#[component]
pub(crate) fn MetricCard(value: String, label: &'static str, tone: &'static str) -> Element {
    rsx! { div { class: "metric-card {tone}", strong { "{value}" } span { "{label}" } } }
}

#[component]
pub(crate) fn BarRow(label: &'static str, value: u8, amount: &'static str) -> Element {
    rsx! { div { class: "bar-row", span { "{label}" } div { class: "bar", div { style: "width: {value}%" } } strong { "{amount}" } } }
}

#[component]
pub(crate) fn PipelineCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    state: &'static str,
    steps: &'static str,
) -> Element {
    rsx! { article { class: "pipeline-card", div { class: "pipeline-icon", "{icon}" } div { class: "pipeline-copy", div { strong { "{title}" } span { class: "badge", "{state}" } } p { "{description}" } small { "{steps}" } } button { class: "secondary", "Open →" } } }
}

#[component]
pub(crate) fn AgentCard(
    initials: &'static str,
    name: &'static str,
    role: &'static str,
    state: &'static str,
    active: bool,
) -> Element {
    rsx! { button { class: if active { "agent-card active" } else { "agent-card" }, span { class: if active { "agent-avatar active" } else { "agent-avatar" }, "{initials}" } div { strong { "{name}" } p { "{role}" } small { "{state}" } } } }
}
