use dioxus::prelude::*;
use polyglid_desktop::{TaskStatus, WorkTrack, WorkspaceOverview};

const MAIN_CSS: &str = include_str!("../assets/main.css");
const PLUGIN_SOURCE: &str = r#"//! Sandboxed first-party diagnostic plugin.

wit_bindgen::generate!({
    world: "security-tool",
    path: "../polyglid-contracts",
});

impl Guest for ReconProbe {
    fn execute(target: String) -> Result<PluginReport, String> {
        let observations = analyze_target(&target, resolve_target(&target));
        build_report(target, observations)
    }
}"#;

fn main() {
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("PolyGlid Developer Space")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(900.0, 620.0));
    let config = dioxus::desktop::Config::new()
        .with_window(window)
        .with_menu(None);
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! { config })
        .launch(App);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceView {
    Explorer,
    Plugins,
    Tracks,
    Automation,
    Agents,
}

impl WorkspaceView {
    fn title(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Plugins => "Plugins",
            Self::Tracks => "Work tracks",
            Self::Automation => "Automation",
            Self::Agents => "AI agents",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EditorTab {
    Scanner,
    Result,
    Source,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BottomTab {
    Problems,
    Output,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsTab {
    Overview,
    Engine,
    Plugins,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrackFilter {
    All,
    Active,
    Planned,
    Complete,
}

impl TrackFilter {
    const ALL: [Self; 4] = [Self::All, Self::Active, Self::Planned, Self::Complete];

    fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Planned => "planned",
            Self::Complete => "complete",
        }
    }

    fn matches(self, status: TaskStatus) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(status, TaskStatus::InProgress | TaskStatus::Review),
            Self::Planned => matches!(status, TaskStatus::Draft | TaskStatus::Ready),
            Self::Complete => matches!(status, TaskStatus::Verified | TaskStatus::Done),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PluginCard {
    id: &'static str,
    name: &'static str,
    version: &'static str,
    description: &'static str,
    capabilities: Vec<&'static str>,
    enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Finding {
    severity: &'static str,
    title: &'static str,
    description: &'static str,
    recommendation: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct ScanReport {
    target: String,
    summary: &'static str,
    findings: Vec<Finding>,
}

#[component]
fn App() -> Element {
    let mut active_view = use_signal(|| WorkspaceView::Explorer);
    let mut editor_tab = use_signal(|| EditorTab::Scanner);
    let mut bottom_tab = use_signal(|| BottomTab::Problems);
    let mut settings_tab = use_signal(|| SettingsTab::Overview);
    let mut settings_open = use_signal(|| false);
    let mut command_open = use_signal(|| false);
    let mut selected_target = use_signal(|| "example.com".to_string());
    let mut new_target = use_signal(String::new);
    let mut targets = use_signal(|| {
        vec![
            "example.com".to_string(),
            "google.com".to_string(),
            "github.com".to_string(),
        ]
    });
    let mut plugins = use_signal(seed_plugins);
    let mut selected_plugin = use_signal(|| "recon-probe".to_string());
    let mut report = use_signal(|| None::<ScanReport>);
    let mut fuel_limit = use_signal(|| 25_000_000_u64);
    let mut track_filter = use_signal(|| TrackFilter::All);
    let mut selected_track = use_signal(|| None::<usize>);
    let overview = use_signal(seed_overview);

    let current_report = report.read().clone();
    let issue_count = current_report
        .as_ref()
        .map_or(0, |value| value.findings.len());

    rsx! {
        style { dangerous_inner_html: MAIN_CSS }
        div { class: "developer-space",
            header { class: "titlebar",
                div { class: "wordmark",
                    span { class: "wordmark-icon", "P" }
                    strong { "polyglid" }
                    span { "/ developer space" }
                }
                button {
                    class: "command-trigger",
                    onclick: move |_| command_open.set(true),
                    span { "Search workspace or run a command" }
                    kbd { "⌘ K" }
                }
                div { class: "window-context",
                    span { class: "live-dot" }
                    "local workspace"
                    button { class: "avatar", "S" }
                }
            }

            div { class: "workspace-body",
                nav { class: "activity-rail", aria_label: "Developer space sections",
                    RailButton { icon: "⌂", label: "Explorer", active: *active_view.read() == WorkspaceView::Explorer, onclick: move |_| active_view.set(WorkspaceView::Explorer) }
                    RailButton { icon: "◇", label: "Plugins", active: *active_view.read() == WorkspaceView::Plugins, onclick: move |_| active_view.set(WorkspaceView::Plugins) }
                    RailButton { icon: "☷", label: "Work tracks", active: *active_view.read() == WorkspaceView::Tracks, onclick: move |_| active_view.set(WorkspaceView::Tracks) }
                    RailButton { icon: "⚙", label: "Automation", active: *active_view.read() == WorkspaceView::Automation, onclick: move |_| active_view.set(WorkspaceView::Automation) }
                    RailButton { icon: "✦", label: "AI agents", active: *active_view.read() == WorkspaceView::Agents, onclick: move |_| active_view.set(WorkspaceView::Agents) }
                    div { class: "rail-spacer" }
                    RailButton { icon: "⚒", label: "Settings", active: false, onclick: move |_| settings_open.set(true) }
                }

                aside { class: "sidebar",
                    div { class: "sidebar-heading",
                        span { "{active_view.read().title()}" }
                        button { "•••" }
                    }
                    match *active_view.read() {
                        WorkspaceView::Explorer => rsx! {
                            div { class: "sidebar-section",
                                p { class: "section-label", "Targets" }
                                div { class: "add-row",
                                    input {
                                        value: "{new_target}",
                                        placeholder: "Add domain or IP",
                                        oninput: move |event| new_target.set(event.value()),
                                    }
                                    button {
                                        onclick: move |_| {
                                            let candidate = new_target.read().trim().to_string();
                                            if !candidate.is_empty() && !targets.read().contains(&candidate) {
                                                targets.write().push(candidate.clone());
                                                selected_target.set(candidate);
                                                new_target.set(String::new());
                                            }
                                        },
                                        "+"
                                    }
                                }
                                div { class: "target-list",
                                    for target in targets.read().iter() {
                                        button {
                                            class: if *selected_target.read() == *target { "target active" } else { "target" },
                                            onclick: {
                                                let target = target.clone();
                                                move |_| {
                                                    selected_target.set(target.clone());
                                                    editor_tab.set(EditorTab::Scanner);
                                                }
                                            },
                                            span { "◎" }
                                            span { "{target}" }
                                        }
                                    }
                                }
                            }
                            div { class: "sidebar-section grow",
                                p { class: "section-label", "Active plugins" }
                                for plugin in plugins.read().iter() {
                                    div { class: if plugin.enabled { "mini-plugin" } else { "mini-plugin disabled" },
                                        span { class: "live-dot" }
                                        div { strong { "{plugin.name}" } small { "{plugin.id}" } }
                                    }
                                }
                            }
                        },
                        WorkspaceView::Plugins => rsx! {
                            div { class: "sidebar-section",
                                p { class: "section-label", "Install component" }
                                input { placeholder: "/path/to/plugin.wasm" }
                                button { class: "primary small", "+ Install plugin" }
                            }
                            div { class: "sidebar-section grow",
                                p { class: "section-label", "Registry" }
                                for plugin in plugins.read().iter() {
                                    button {
                                        class: if selected_plugin.read().as_str() == plugin.id { "plugin-nav active" } else { "plugin-nav" },
                                        onclick: {
                                            let id = plugin.id.to_string();
                                            move |_| selected_plugin.set(id.clone())
                                        },
                                        span { class: if plugin.enabled { "live-dot" } else { "live-dot off" } }
                                        div { strong { "{plugin.name}" } small { "v{plugin.version}" } }
                                    }
                                }
                            }
                        },
                        WorkspaceView::Tracks => rsx! {
                            div { class: "sidebar-section grow",
                                p { class: "section-label", "Delivery filters" }
                                for filter in TrackFilter::ALL {
                                    button {
                                        class: if *track_filter.read() == filter { "sidebar-option active" } else { "sidebar-option" },
                                        onclick: move |_| track_filter.set(filter),
                                        span { "{filter.label()}" }
                                        small { "{track_count(&overview.read(), filter)}" }
                                    }
                                }
                            }
                        },
                        WorkspaceView::Automation => rsx! {
                            div { class: "sidebar-section grow",
                                p { class: "section-label", "Pipelines" }
                                SidebarOption { label: "Workspace verify", meta: "ready", active: true }
                                SidebarOption { label: "Rust quality", meta: "4 steps", active: false }
                                SidebarOption { label: "Security review", meta: "3 steps", active: false }
                                SidebarOption { label: "Release gate", meta: "draft", active: false }
                            }
                        },
                        WorkspaceView::Agents => rsx! {
                            div { class: "sidebar-section grow",
                                p { class: "section-label", "Agent roster" }
                                SidebarOption { label: "Executive", meta: "online", active: true }
                                SidebarOption { label: "Code analyst", meta: "idle", active: false }
                                SidebarOption { label: "Security reviewer", meta: "idle", active: false }
                                SidebarOption { label: "Test helper", meta: "idle", active: false }
                            }
                        },
                    }
                }

                div { class: "main-column",
                    main { class: "editor",
                        match *active_view.read() {
                            WorkspaceView::Explorer => rsx! {
                                div { class: "editor-tabs",
                                    EditorTabButton { label: "Scanner configuration", icon: "⚡", active: *editor_tab.read() == EditorTab::Scanner, onclick: move |_| editor_tab.set(EditorTab::Scanner) }
                                    if current_report.is_some() {
                                        EditorTabButton { label: "Result dashboard", icon: "▥", active: *editor_tab.read() == EditorTab::Result, onclick: move |_| editor_tab.set(EditorTab::Result) }
                                    }
                                    EditorTabButton { label: "recon_probe.rs", icon: "Rs", active: *editor_tab.read() == EditorTab::Source, onclick: move |_| editor_tab.set(EditorTab::Source) }
                                }
                                match *editor_tab.read() {
                                    EditorTab::Scanner => rsx! { ScannerDashboard {
                                        target: selected_target.read().clone(),
                                        selected_plugin: selected_plugin.read().clone(),
                                        plugins: plugins.read().clone(),
                                        on_target: move |value| selected_target.set(value),
                                        on_plugin: move |value| selected_plugin.set(value),
                                        on_run: move |_| {
                                            report.set(Some(sample_report(selected_target.read().clone())));
                                            editor_tab.set(EditorTab::Result);
                                            bottom_tab.set(BottomTab::Problems);
                                        }
                                    } },
                                    EditorTab::Result => rsx! { ResultDashboard { report: current_report.clone() } },
                                    EditorTab::Source => rsx! { SourceDashboard {} },
                                }
                            },
                            WorkspaceView::Plugins => rsx! { PluginDashboard {
                                plugins: plugins.read().clone(),
                                selected: selected_plugin.read().clone(),
                                on_toggle: move |id: String| {
                                    if let Some(plugin) = plugins.write().iter_mut().find(|plugin| plugin.id == id) {
                                        plugin.enabled = !plugin.enabled;
                                    }
                                }
                            } },
                            WorkspaceView::Tracks => rsx! { TracksDashboard {
                                overview: overview.read().clone(),
                                filter: *track_filter.read(),
                                selected: *selected_track.read(),
                                on_select: move |index| selected_track.set(index),
                            } },
                            WorkspaceView::Automation => rsx! { AutomationDashboard {} },
                            WorkspaceView::Agents => rsx! { AgentsDashboard {} },
                        }
                    }

                    section { class: "bottom-panel",
                        div { class: "bottom-tabs",
                            BottomTabButton { label: "Problems", count: Some(issue_count), active: *bottom_tab.read() == BottomTab::Problems, onclick: move |_| bottom_tab.set(BottomTab::Problems) }
                            BottomTabButton { label: "Output", count: None, active: *bottom_tab.read() == BottomTab::Output, onclick: move |_| bottom_tab.set(BottomTab::Output) }
                            BottomTabButton { label: "Terminal", count: None, active: *bottom_tab.read() == BottomTab::Terminal, onclick: move |_| bottom_tab.set(BottomTab::Terminal) }
                            div { class: "panel-actions", "⌃  □  ×" }
                        }
                        div { class: "bottom-content",
                            match *bottom_tab.read() {
                                BottomTab::Problems => rsx! { ProblemsPanel { report: current_report.clone() } },
                                BottomTab::Output => rsx! {
                                    div { class: "console",
                                        p { span { class: "dim", "[info]" } " control plane initialized" }
                                        p { span { class: "dim", "[info]" } " Wasmtime sandbox ready · fuel {fuel_limit}" }
                                        p { span { class: "success", "[ready]" } " local workspace indexed" }
                                        if let Some(value) = current_report.as_ref() {
                                            p { span { class: "accent", "[scan]" } " recon-probe completed for {value.target}" }
                                        }
                                    }
                                },
                                BottomTab::Terminal => rsx! {
                                    div { class: "console terminal",
                                        p { class: "dim", "PolyGlid interactive host shell" }
                                        p { "polyglid workspace verify" }
                                        p { class: "success", "✓ contracts  ✓ permissions  ✓ runtime" }
                                        p { span { class: "prompt", "❯" } " _" }
                                    }
                                },
                            }
                        }
                    }
                }
            }

            footer { class: "statusbar",
                div { span { "◈" } " PolyGlid Core Ready" }
                div { span { "◉" } " Wasmtime Engine" }
                div { class: "status-spacer" }
                div { "Fuel: {fuel_limit}" }
                div { "Plugins: {plugins.read().iter().filter(|plugin| plugin.enabled).count()}" }
                div { "Rust · local" }
            }

            if *settings_open.read() {
                div { class: "modal-backdrop", onclick: move |_| settings_open.set(false),
                    div { class: "settings-modal", onclick: move |event| event.stop_propagation(),
                        div { class: "modal-header",
                            strong { "⚒ PolyGlid settings" }
                            button { onclick: move |_| settings_open.set(false), "×" }
                        }
                        div { class: "modal-body",
                            nav { class: "settings-nav",
                                SettingsButton { label: "Overview", active: *settings_tab.read() == SettingsTab::Overview, onclick: move |_| settings_tab.set(SettingsTab::Overview) }
                                SettingsButton { label: "Engine", active: *settings_tab.read() == SettingsTab::Engine, onclick: move |_| settings_tab.set(SettingsTab::Engine) }
                                SettingsButton { label: "Plugins", active: *settings_tab.read() == SettingsTab::Plugins, onclick: move |_| settings_tab.set(SettingsTab::Plugins) }
                            }
                            div { class: "settings-content",
                                match *settings_tab.read() {
                                    SettingsTab::Overview => rsx! { SettingsOverview {} },
                                    SettingsTab::Engine => rsx! {
                                        h2 { "WASM engine" }
                                        p { class: "muted", "Configure safety thresholds for local component execution." }
                                        label { class: "field-label", "Maximum WASM fuel" }
                                        input {
                                            r#type: "number",
                                            value: "{fuel_limit}",
                                            oninput: move |event| {
                                                if let Ok(value) = event.value().parse() { fuel_limit.set(value); }
                                            }
                                        }
                                        p { class: "field-help", "Prevents CPU starvation and infinite guest loops." }
                                    },
                                    SettingsTab::Plugins => rsx! {
                                        h2 { "Loaded plugins" }
                                        p { class: "muted", "Workspace components and their current runtime state." }
                                        for plugin in plugins.read().iter() {
                                            div { class: "setting-row",
                                                div { strong { "{plugin.name}" } small { "{plugin.id} · v{plugin.version}" } }
                                                span { class: if plugin.enabled { "badge good" } else { "badge" }, if plugin.enabled { "Enabled" } else { "Disabled" } }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        div { class: "modal-footer", button { class: "primary small", onclick: move |_| settings_open.set(false), "Done" } }
                    }
                }
            }

            if *command_open.read() {
                div { class: "modal-backdrop command-backdrop", onclick: move |_| command_open.set(false),
                    div { class: "command-palette", onclick: move |event| event.stop_propagation(),
                        input { autofocus: true, placeholder: "Type a command or search the workspace…" }
                        p { class: "section-label", "Quick navigation" }
                        button { onclick: move |_| { active_view.set(WorkspaceView::Explorer); command_open.set(false); }, "⚡ Open scanner" span { "Explorer" } }
                        button { onclick: move |_| { active_view.set(WorkspaceView::Tracks); command_open.set(false); }, "☷ Open work tracks" span { "Project" } }
                        button { onclick: move |_| { active_view.set(WorkspaceView::Automation); command_open.set(false); }, "⚙ Run workspace verification" span { "Automation" } }
                    }
                }
            }
        }
    }
}

#[component]
fn RailButton(
    icon: &'static str,
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "rail-button active" } else { "rail-button" }, title: label, onclick: move |event| onclick.call(event), span { "{icon}" } small { "{label}" } } }
}

#[component]
fn SidebarOption(label: &'static str, meta: &'static str, active: bool) -> Element {
    rsx! { button { class: if active { "sidebar-option active" } else { "sidebar-option" }, span { "{label}" } small { "{meta}" } } }
}

#[component]
fn EditorTabButton(
    label: &'static str,
    icon: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "editor-tab active" } else { "editor-tab" }, onclick: move |event| onclick.call(event), span { "{icon}" } "{label}" } }
}

#[component]
fn BottomTabButton(
    label: &'static str,
    count: Option<usize>,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: if active { "bottom-tab active" } else { "bottom-tab" }, onclick: move |event| onclick.call(event), "{label}" if let Some(value) = count { span { "{value}" } } } }
}

#[component]
fn SettingsButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! { button { class: if active { "active" } else { "" }, onclick: move |event| onclick.call(event), "{label}" } }
}

#[component]
fn ScannerDashboard(
    target: String,
    selected_plugin: String,
    plugins: Vec<PluginCard>,
    on_target: EventHandler<String>,
    on_plugin: EventHandler<String>,
    on_run: EventHandler<MouseEvent>,
) -> Element {
    let selected_enabled = plugins
        .iter()
        .find(|plugin| plugin.id == selected_plugin)
        .is_none_or(|plugin| plugin.enabled);
    rsx! {
        div { class: "dashboard-scroll scanner-page",
            div { class: "page-heading centered",
                span { class: "eyebrow", "Sandboxed execution" }
                h1 { "Security scanner" }
                p { "Configure a target and launch a permission-controlled WebAssembly component." }
            }
            div { class: "scanner-card",
                label { class: "field-label", "Target domain or IP" }
                input { value: "{target}", placeholder: "example.com", oninput: move |event| on_target.call(event.value()) }
                label { class: "field-label", "Selected plugin" }
                select { value: "{selected_plugin}", onchange: move |event| on_plugin.call(event.value()),
                    for plugin in plugins { option { value: "{plugin.id}", "{plugin.name} · {plugin.id}" } }
                }
                div { class: "permission-strip",
                    span { "◈ WASI sandbox" }
                    span { "◎ scoped DNS" }
                    span { "▣ report write" }
                }
                if !selected_enabled { p { class: "error-text", "This plugin is disabled. Enable it from Plugin management." } }
                button { class: "primary run-button", disabled: !selected_enabled, onclick: move |event| on_run.call(event), "▶ Run analysis" }
            }
        }
    }
}

#[component]
fn ResultDashboard(report: Option<ScanReport>) -> Element {
    rsx! {
        div { class: "dashboard-scroll result-page",
            if let Some(value) = report {
                div { class: "result-hero",
                    div { span { class: "eyebrow", "Analysis complete" } h1 { "{value.target}" } p { "{value.summary}" } }
                    div { class: "risk-score", strong { "82" } span { "health score" } }
                }
                div { class: "metric-grid three",
                    MetricCard { value: value.findings.len().to_string(), label: "observations", tone: "warning" }
                    MetricCard { value: "0".to_string(), label: "critical", tone: "good" }
                    MetricCard { value: "148ms".to_string(), label: "runtime", tone: "neutral" }
                }
                div { class: "chart-card",
                    div { class: "card-heading", strong { "Finding distribution" } span { "by confidence" } }
                    BarRow { label: "Informational", value: 74, amount: "74%" }
                    BarRow { label: "Low", value: 42, amount: "42%" }
                    BarRow { label: "Medium", value: 18, amount: "18%" }
                }
            } else {
                div { class: "empty-state", "Run an analysis to create a result dashboard." }
            }
        }
    }
}

#[component]
fn SourceDashboard() -> Element {
    rsx! { div { class: "source-view", div { class: "source-note", "Read-only · projects/recon-probe/src/lib.rs" } pre { code { "{PLUGIN_SOURCE}" } } } }
}

#[component]
fn PluginDashboard(
    plugins: Vec<PluginCard>,
    selected: String,
    on_toggle: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Component registry" } h1 { "Plugin management" } p { "Inspect capabilities and control which signed components may execute." } }
            div { class: "plugin-grid",
                for plugin in plugins {
                    article { class: if plugin.id == selected { "plugin-card selected" } else { "plugin-card" },
                        div { class: "plugin-card-head", span { class: if plugin.enabled { "plugin-symbol enabled" } else { "plugin-symbol" }, "◇" } span { class: if plugin.enabled { "badge good" } else { "badge" }, if plugin.enabled { "Enabled" } else { "Disabled" } } }
                        h2 { "{plugin.name}" }
                        p { "{plugin.description}" }
                        small { "{plugin.id} · v{plugin.version}" }
                        div { class: "capability-list", for cap in &plugin.capabilities { span { "{cap}" } } }
                        button { class: "secondary", onclick: { let id = plugin.id.to_string(); move |_| on_toggle.call(id.clone()) }, if plugin.enabled { "Disable component" } else { "Enable component" } }
                    }
                }
            }
        }
    }
}

#[component]
fn TracksDashboard(
    overview: WorkspaceOverview,
    filter: TrackFilter,
    selected: Option<usize>,
    on_select: EventHandler<Option<usize>>,
) -> Element {
    let visible: Vec<_> = overview
        .tracks
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, track)| filter.matches(track.status))
        .collect();
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Execution curriculum" } h1 { "Work tracks" } p { "Build the platform in ordered, verifiable stages with visible evidence and ownership." } }
            div { class: "metric-grid four",
                MetricCard { value: overview.tracks.len().to_string(), label: "tracks", tone: "neutral" }
                MetricCard { value: overview.total_tasks().to_string(), label: "tasks", tone: "neutral" }
                MetricCard { value: overview.active_tracks().to_string(), label: "active", tone: "good" }
                MetricCard { value: overview.completed_tasks().to_string(), label: "completed", tone: "neutral" }
            }
            div { class: "track-list",
                for (index, track) in visible {
                    article { class: if selected == Some(index) { "track-card selected" } else { "track-card" },
                        button { class: "track-main", onclick: move |_| on_select.call(if selected == Some(index) { None } else { Some(index) }),
                            span { class: "track-number", "{track.sequence:02}" }
                            div { class: "track-body", div { class: "track-meta", span { "Platform track" } span { class: "badge", "{track.status.label()}" } } h2 { "{track.title}" } p { "{track.description}" } div { class: "capability-list", for tag in &track.tags { span { "#{tag}" } } } div { class: "progress-label", span { "{track.completed_tasks} / {track.total_tasks} tasks" } span { "{track.progress_percent()}%" } } div { class: "progress", div { style: "width: {track.progress_percent()}%" } } }
                            span { class: "track-arrow", if selected == Some(index) { "−" } else { "→" } }
                        }
                        if selected == Some(index) { div { class: "track-detail", span { "Next action" } strong { "Open the track plan and complete its verification gate." } button { class: "secondary", "View tasks" } } }
                    }
                }
            }
        }
    }
}

#[component]
fn AutomationDashboard() -> Element {
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Deterministic execution" } h1 { "Automation control" } p { "Run typed workspace workflows and keep evidence for every command." } }
            div { class: "metric-grid four", MetricCard { value: "4".to_string(), label: "pipelines", tone: "neutral" } MetricCard { value: "12".to_string(), label: "checks", tone: "neutral" } MetricCard { value: "11".to_string(), label: "passing", tone: "good" } MetricCard { value: "1".to_string(), label: "waiting", tone: "warning" } }
            div { class: "pipeline-grid",
                PipelineCard { icon: "✓", title: "Workspace verify", description: "Contracts, permissions, formatting, tests, and dependency boundaries.", state: "Ready", steps: "6 steps" }
                PipelineCard { icon: "Rs", title: "Rust quality", description: "Format, check, test, Clippy, audit, and build evidence.", state: "Passing", steps: "4 steps" }
                PipelineCard { icon: "◈", title: "Security review", description: "Threat review, unsafe scan, dependency audit, and signed report.", state: "Review", steps: "3 steps" }
                PipelineCard { icon: "↗", title: "Release gate", description: "Package artifacts only after required evidence and approvals exist.", state: "Draft", steps: "5 steps" }
            }
        }
    }
}

#[component]
fn AgentsDashboard() -> Element {
    rsx! {
        div { class: "dashboard-scroll agents-page",
            div { class: "page-heading", span { class: "eyebrow", "Human-controlled assistance" } h1 { "AI agent workspace" } p { "Delegate bounded work while the executive preserves context, approvals, and evidence." } }
            div { class: "agent-layout",
                section { class: "agent-roster",
                    AgentCard { initials: "EX", name: "Executive", role: "Plans, routes, and verifies work", state: "Online", active: true }
                    AgentCard { initials: "CA", name: "Code analyst", role: "Maps code and explains behavior", state: "Idle", active: false }
                    AgentCard { initials: "SR", name: "Security reviewer", role: "Finds risks and validates boundaries", state: "Idle", active: false }
                    AgentCard { initials: "TH", name: "Test helper", role: "Runs focused verification", state: "Idle", active: false }
                }
                section { class: "agent-console",
                    div { class: "agent-console-head", div { span { class: "agent-avatar active", "EX" } strong { "Executive agent" } } span { class: "badge good", "Ready" } }
                    div { class: "conversation",
                        div { class: "message system", span { "System" } p { "Workspace context loaded: architecture, plans, recent changes, and verification rules." } }
                        div { class: "message assistant", span { "Executive" } p { "Choose a bounded task. I will create a plan, route specialist work, and require evidence before completion." } }
                    }
                    div { class: "suggestions", button { "Analyze current phase" } button { "Review security" } button { "Prepare next task" } }
                    div { class: "composer", textarea { placeholder: "Describe the outcome you want…" } button { "Send ↗" } }
                }
            }
        }
    }
}

#[component]
fn ProblemsPanel(report: Option<ScanReport>) -> Element {
    rsx! {
        if let Some(value) = report {
            div { class: "problems-list",
                for finding in value.findings {
                    div { class: "finding", span { class: "finding-icon", "!" } div { div { strong { "{finding.title}" } span { class: "badge", "{finding.severity}" } } p { "{finding.description}" } small { "→ {finding.recommendation}" } } }
                }
            }
        } else { div { class: "panel-empty", "No active findings. Choose a target and run analysis." } }
    }
}

#[component]
fn MetricCard(value: String, label: &'static str, tone: &'static str) -> Element {
    rsx! { div { class: "metric-card {tone}", strong { "{value}" } span { "{label}" } } }
}

#[component]
fn BarRow(label: &'static str, value: u8, amount: &'static str) -> Element {
    rsx! { div { class: "bar-row", span { "{label}" } div { class: "bar", div { style: "width: {value}%" } } strong { "{amount}" } } }
}

#[component]
fn PipelineCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    state: &'static str,
    steps: &'static str,
) -> Element {
    rsx! { article { class: "pipeline-card", div { class: "pipeline-icon", "{icon}" } div { class: "pipeline-copy", div { strong { "{title}" } span { class: "badge", "{state}" } } p { "{description}" } small { "{steps}" } } button { class: "secondary", "Open →" } } }
}

#[component]
fn AgentCard(
    initials: &'static str,
    name: &'static str,
    role: &'static str,
    state: &'static str,
    active: bool,
) -> Element {
    rsx! { button { class: if active { "agent-card active" } else { "agent-card" }, span { class: if active { "agent-avatar active" } else { "agent-avatar" }, "{initials}" } div { strong { "{name}" } p { "{role}" } small { "{state}" } } } }
}

#[component]
fn SettingsOverview() -> Element {
    rsx! { h2 { "System overview" } p { class: "muted", "Status of the local sandbox and control plane." } div { class: "settings-grid", div { class: "setting-card", span { "Engine runtime" } strong { "◉ Wasmtime 46" } } div { class: "setting-card", span { "Sandbox model" } strong { "◈ WASI Preview 1" } } } h3 { "Active capabilities" } div { class: "setting-row", code { "dns-resolve" } span { class: "badge good", "Scoped" } } div { class: "setting-row", code { "report-write" } span { class: "badge good", "Scoped" } } }
}

fn track_count(overview: &WorkspaceOverview, filter: TrackFilter) -> usize {
    overview
        .tracks
        .iter()
        .filter(|track| filter.matches(track.status))
        .count()
}

fn seed_plugins() -> Vec<PluginCard> {
    vec![
        PluginCard {
            id: "recon-probe",
            name: "Recon Probe",
            version: "0.1.0",
            description: "Safe DNS and target diagnostics rendered through native panels.",
            capabilities: vec!["dns-resolve", "report-write"],
            enabled: true,
        },
        PluginCard {
            id: "dependency-audit",
            name: "Dependency Audit",
            version: "0.1.0",
            description: "Reviews Rust dependency metadata and produces bounded findings.",
            capabilities: vec!["workspace-read"],
            enabled: true,
        },
        PluginCard {
            id: "code-review",
            name: "Code Review",
            version: "0.1.0",
            description: "Local-first static review helper behind explicit approval gates.",
            capabilities: vec!["workspace-read", "ai-inference"],
            enabled: false,
        },
    ]
}

fn sample_report(target: String) -> ScanReport {
    ScanReport {
        target,
        summary:
            "The target responded normally. Two low-risk configuration observations need review.",
        findings: vec![
            Finding {
                severity: "LOW",
                title: "DNS response exposes multiple addresses",
                description: "The target resolves through more than one public endpoint.",
                recommendation: "Confirm every endpoint belongs to the intended deployment.",
            },
            Finding {
                severity: "INFO",
                title: "Report evidence created",
                description:
                    "The sandbox wrote a structured report through the scoped host capability.",
                recommendation: "Review and sign the evidence before sharing it.",
            },
        ],
    }
}

fn seed_overview() -> WorkspaceOverview {
    WorkspaceOverview { tracks: vec![
        WorkTrack::new(1, "Foundation", "Versioned contracts, domain rules, state transitions, and a runnable control-plane shell.", TaskStatus::InProgress, 3, 6, ["contracts", "domain", "dioxus"]),
        WorkTrack::new(2, "Local persistence", "SQLite repositories, migrations, audit history, transactional outbox, and recovery.", TaskStatus::Ready, 0, 7, ["sqlite", "sqlx", "audit"]),
        WorkTrack::new(3, "Git collaboration", "Immutable WPM events synchronized through an isolated data branch.", TaskStatus::Draft, 0, 8, ["git", "events", "offline"]),
        WorkTrack::new(4, "Automation engine", "Typed validation, build, test, reports, and execution evidence.", TaskStatus::Draft, 0, 9, ["executor", "cargo", "evidence"]),
        WorkTrack::new(5, "AI assistance", "Local-first search, analysis, generation, and review behind approval gates.", TaskStatus::Draft, 0, 10, ["agents", "rag", "security"]),
    ].into_iter().map(|track| track.expect("seed track is valid")).collect() }
}
