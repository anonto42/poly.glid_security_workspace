use dioxus::prelude::*;

const STYLES: Asset = asset!("/assets/styles.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: STYLES }
        document::Meta {
            name: "description",
            content: "PolyGlid is a local-first security workspace built in Rust with explicit safety boundaries.",
        }

        div { class: "page-shell",
            Header {}
            main {
                Hero {}
                Principles {}
                ProjectStatus {}
            }
            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "site-header",
            a {
                class: "brand",
                href: "#top",
                aria_label: "PolyGlid home",
                span { class: "brand-mark", "P/G" }
                span { "POLYGLID" }
            }

            nav { aria_label: "Primary navigation",
                a { href: "#principles", "Principles" }
                a { href: "#status", "Status" }
                a {
                    class: "nav-action",
                    href: "https://github.com/anonto42/polyglid",
                    target: "_blank",
                    rel: "noreferrer",
                    "Source ↗"
                }
            }
        }
    }
}

#[component]
fn Hero() -> Element {
    rsx! {
        section { class: "hero", id: "top",
            div { class: "hero-copy",
                div { class: "eyebrow",
                    span { class: "status-pulse" }
                    "Projects phase · local-first"
                }
                h1 {
                    "A security workspace "
                    span { "designed for control." }
                }
                p { class: "hero-lede",
                    "Organize local projects in one deliberate desktop environment. "
                    "PolyGlid keeps operations visible, permissions explicit, and your workspace on your machine."
                }
                div { class: "hero-actions",
                    a {
                        class: "button button-primary",
                        href: "https://github.com/anonto42/polyglid/releases",
                        "View releases"
                        span { aria_hidden: "true", "→" }
                    }
                    a {
                        class: "button button-secondary",
                        href: "https://github.com/anonto42/polyglid",
                        "Explore the code"
                    }
                }
            }

            WorkspacePreview {}
        }
    }
}

#[component]
fn WorkspacePreview() -> Element {
    rsx! {
        div { class: "workspace-card", aria_label: "PolyGlid workspace preview",
            div { class: "workspace-topbar",
                div { class: "window-controls",
                    span {}
                    span {}
                    span {}
                }
                span { "workspace / projects" }
                span { class: "secure-label", "LOCAL" }
            }
            div { class: "workspace-body",
                aside {
                    span { class: "rail-label", "WORKSPACES" }
                    div { class: "rail-item rail-item-active",
                        span { class: "rail-glyph", "A" }
                        div {
                            strong { "Aloevol" }
                            small { "3 projects" }
                        }
                    }
                    div { class: "rail-item",
                        span { class: "rail-glyph", "L" }
                        div {
                            strong { "Lab" }
                            small { "1 project" }
                        }
                    }
                }
                div { class: "project-panel",
                    div { class: "panel-heading",
                        div {
                            small { "ACTIVE WORKSPACE" }
                            strong { "Aloevol security" }
                        }
                        span { "+ New project" }
                    }
                    div { class: "project-row selected",
                        span { class: "project-index", "01" }
                        div {
                            strong { "polyglid" }
                            small { "~/workspace/polyglid" }
                        }
                        span { class: "project-state", "ACTIVE" }
                    }
                    div { class: "project-row",
                        span { class: "project-index", "02" }
                        div {
                            strong { "runtime-lab" }
                            small { "~/workspace/runtime-lab" }
                        }
                        span { class: "project-state muted", "READY" }
                    }
                    div { class: "project-row",
                        span { class: "project-index", "03" }
                        div {
                            strong { "plugin-research" }
                            small { "~/workspace/plugin-research" }
                        }
                        span { class: "project-state muted", "READY" }
                    }
                }
            }
            div { class: "workspace-footer",
                span { "Data stays on this device" }
                span { "Rust · Dioxus · SQLite" }
            }
        }
    }
}

#[component]
fn Principles() -> Element {
    let principles = [
        (
            "01",
            "Local by default",
            "Projects, preferences, and workspace state remain under your control instead of depending on a hosted account.",
        ),
        (
            "02",
            "Explicit boundaries",
            "Sensitive actions are designed around visible intent, scoped capabilities, and confirmation at the point of risk.",
        ),
        (
            "03",
            "One Rust foundation",
            "The desktop product and this website use Dioxus, keeping the interface model consistent across native and web surfaces.",
        ),
    ];

    rsx! {
        section { class: "principles section-wrap", id: "principles",
            div { class: "section-kicker", "DESIGN PRINCIPLES" }
            div { class: "section-intro",
                h2 { "Security tools should feel precise, not mysterious." }
                p {
                    "PolyGlid is being assembled one dependable product phase at a time. "
                    "The foundation favors clarity over hidden automation."
                }
            }
            div { class: "principle-grid",
                for (number, title, description) in principles {
                    article { class: "principle-card",
                        span { class: "principle-number", "{number}" }
                        h3 { "{title}" }
                        p { "{description}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ProjectStatus() -> Element {
    rsx! {
        section { class: "status-section section-wrap", id: "status",
            div { class: "status-copy",
                div { class: "section-kicker", "CURRENT BUILD" }
                h2 { "The foundation is intentionally focused." }
                p {
                    "Today, PolyGlid manages local workspaces and projects. Runtime, plugin, "
                    "reporting, and collaboration interfaces will return only when their product phases are ready."
                }
            }
            div { class: "status-board",
                div { class: "status-row status-ready",
                    span { "Workspace discovery" }
                    strong { "READY" }
                }
                div { class: "status-row status-ready",
                    span { "Project lifecycle" }
                    strong { "READY" }
                }
                div { class: "status-row",
                    span { "Security execution UI" }
                    strong { "PLANNED" }
                }
                div { class: "status-row",
                    span { "Plugin marketplace" }
                    strong { "PLANNED" }
                }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer {
            div { class: "footer-brand",
                span { class: "brand-mark", "P/G" }
                div {
                    strong { "PolyGlid" }
                    small { "Built deliberately in Rust." }
                }
            }
            div { class: "footer-links",
                a {
                    href: "https://github.com/anonto42/polyglid",
                    "GitHub"
                }
                a {
                    href: "https://github.com/anonto42/polyglid/blob/main/LICENSE-MIT",
                    "License"
                }
            }
        }
    }
}
