use polyglid_desktop::{TaskStatus, WorkTrack, WorkspaceOverview};

use super::models::{PluginCard, TopBarAction, WorkspaceView};

pub(crate) const PLUGIN_SOURCE: &str = r#"//! Sandboxed first-party diagnostic plugin.

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

pub(crate) fn seed_plugins() -> Vec<PluginCard> {
    vec![
        PluginCard {
            id: "recon-probe".into(),
            name: "Recon Probe".into(),
            version: "0.1.0".into(),
            description: "Safe DNS and target diagnostics rendered through native panels.".into(),
            capabilities: vec!["dns-resolve".into(), "report-write".into()],
            enabled: true,
        },
        PluginCard {
            id: "dependency-audit".into(), name: "Dependency Audit".into(), version: "0.1.0".into(),
            description: "Reviews Rust dependency metadata and produces bounded findings.".into(), capabilities: vec!["workspace-read".into()],
            enabled: true,
        },
        PluginCard {
            id: "code-review".into(), name: "Code Review".into(), version: "0.1.0".into(),
            description: "Local-first static review helper behind explicit approval gates.".into(), capabilities: vec!["workspace-read".into(), "ai-inference".into()],
            enabled: false,
        },
    ]
}

pub(crate) fn seed_top_bar_actions() -> Vec<TopBarAction> {
    vec![
        TopBarAction {
            id: "recon-probe.open",
            icon: "◇",
            label: "Open plugins",
            source: "Recon Probe",
            destination: WorkspaceView::Plugins,
        },
        TopBarAction {
            id: "workspace.verify",
            icon: "✓",
            label: "Workspace verify",
            source: "PolyGlid Core",
            destination: WorkspaceView::Automation,
        },
    ]
}

pub(crate) fn seed_overview() -> WorkspaceOverview {
    let tracks = vec![
        WorkTrack::new(1, "Foundation", "Versioned contracts, domain rules, state transitions, and a runnable control-plane shell.", TaskStatus::InProgress, 3, 6, ["contracts", "domain", "dioxus"]),
        WorkTrack::new(2, "Local persistence", "SQLite repositories, migrations, audit history, transactional outbox, and recovery.", TaskStatus::Ready, 0, 7, ["sqlite", "sqlx", "audit"]),
        WorkTrack::new(3, "Git collaboration", "Immutable WPM events synchronized through an isolated data branch.", TaskStatus::Draft, 0, 8, ["git", "events", "offline"]),
        WorkTrack::new(4, "Automation engine", "Typed validation, build, test, reports, and execution evidence.", TaskStatus::Draft, 0, 9, ["executor", "cargo", "evidence"]),
        WorkTrack::new(5, "AI assistance", "Local-first search, analysis, generation, and review behind approval gates.", TaskStatus::Draft, 0, 10, ["agents", "rag", "security"]),
    ];
    WorkspaceOverview {
        tracks: tracks
            .into_iter()
            .map(|track| track.expect("valid preview track"))
            .collect(),
    }
}
