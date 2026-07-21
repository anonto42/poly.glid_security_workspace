//! Harmless first-party demo plugin logic.
//!
//! This plugin is a self-contained WASM component that exports:
//! - `execute(target)` — the security analysis logic
//! - `metadata()` — embedded plugin description
//! - `required_capabilities()` — capability requirements
//! - `cli_panel(report)` — structured TUI layout for results
//! - `desktop_panel(report)` — structured desktop layout for results

wit_bindgen::generate!({
    world: "security-tool",
    path: "../../contracts/wit",
});

use crate::polyglid::engine::{
    dns, reports,
    types::{Issue, PanelWidget, Severity, WidgetType},
};

struct ReconProbe;

impl Guest for ReconProbe {
    fn execute(target: String) -> Result<PluginReport, String> {
        let mut observations = analyze_target(&target, resolve_target(&target));
        observations.extend(
            write_summary_report(&target)
                .err()
                .map(|message| ReconObservation {
                    title: "Report write unavailable".to_string(),
                    description: format!(
                        "The report-write capability did not store a summary: {message}"
                    ),
                    recommendation: "Approve report-write for the configured reports directory."
                        .to_string(),
                }),
        );

        let issues = observations
            .into_iter()
            .map(|observation| Issue {
                title: observation.title,
                severity: Severity::Info,
                description: observation.description,
                recommendation: observation.recommendation,
            })
            .collect::<Vec<_>>();

        let summary = if issues.is_empty() {
            format!("No demo observations were reported for {target}.")
        } else {
            format!(
                "{} demo observation(s) reported for {target}.",
                issues.len()
            )
        };

        Ok(PluginReport {
            plugin_name: "PolyGlid Recon Probe".to_string(),
            target_tested: target,
            issues,
            summary,
        })
    }

    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "recon_probe".to_string(),
            display_name: "Reconnaissance Probe".to_string(),
            version: "0.1.0".to_string(),
            description: "Performs secure DNS resolutions and reports network structure."
                .to_string(),
            author: "PolyGlid Core Team".to_string(),
        }
    }

    fn required_capabilities() -> Vec<String> {
        vec!["dns-resolve".to_string(), "report-write".to_string()]
    }

    fn cli_panel(report: PluginReport) -> PanelLayout {
        let mut widgets = Vec::new();

        // Issues table
        let mut table_data = vec![vec![
            "Severity".to_string(),
            "Title".to_string(),
            "Description".to_string(),
        ]];
        for issue in &report.issues {
            table_data.push(vec![
                format!("{:?}", issue.severity),
                issue.title.clone(),
                issue.description.clone(),
            ]);
        }
        widgets.push(PanelWidget {
            widget_kind: WidgetType::Table,
            title: "Security Observations".to_string(),
            data: table_data,
        });

        // Summary key-value
        widgets.push(PanelWidget {
            widget_kind: WidgetType::KeyValue,
            title: "Scan Summary".to_string(),
            data: vec![
                vec!["Plugin".to_string(), report.plugin_name.clone()],
                vec!["Target".to_string(), report.target_tested.clone()],
                vec!["Issues Found".to_string(), report.issues.len().to_string()],
                vec!["Summary".to_string(), report.summary.clone()],
            ],
        });

        // Recommendations
        let rec_data: Vec<Vec<String>> = report
            .issues
            .iter()
            .map(|issue| vec![issue.recommendation.clone()])
            .collect();
        if !rec_data.is_empty() {
            widgets.push(PanelWidget {
                widget_kind: WidgetType::TextBlock,
                title: "Recommendations".to_string(),
                data: rec_data,
            });
        }

        PanelLayout {
            title: "Recon Probe Results".to_string(),
            widgets,
        }
    }

    fn desktop_panel(report: PluginReport) -> PanelLayout {
        // Desktop uses the same layout as CLI for now.
        // Plugin authors can customize this for richer rendering.
        Self::cli_panel(report)
    }
}

fn resolve_target(target: &str) -> Result<Vec<String>, String> {
    dns::resolve(target)
}

fn write_summary_report(target: &str) -> Result<String, String> {
    let filename = format!("recon-probe-{}.txt", report_safe_target(target));
    let contents = format!("PolyGlid Recon Probe\nTarget: {target}\n");
    reports::write(&filename, &contents)
}

fn report_safe_target(target: &str) -> String {
    target
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

export!(ReconProbe);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconObservation {
    pub title: String,
    pub description: String,
    pub recommendation: String,
}

pub fn analyze_target(
    target: &str,
    resolution: Result<Vec<String>, String>,
) -> Vec<ReconObservation> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return vec![ReconObservation {
            title: "Empty target".to_string(),
            description: "The provided target was blank.".to_string(),
            recommendation: "Provide an explicit host, domain, or URL.".to_string(),
        }];
    }

    if matches!(trimmed, "localhost" | "127.0.0.1" | "::1") {
        return vec![ReconObservation {
            title: "Loopback target".to_string(),
            description: "The target points at the local machine.".to_string(),
            recommendation: "Use loopback only for local defensive tests.".to_string(),
        }];
    }

    match resolution {
        Ok(addresses) if addresses.is_empty() => vec![ReconObservation {
            title: "DNS returned no addresses".to_string(),
            description: format!(
                "The host resolved successfully but no addresses were returned for {trimmed}."
            ),
            recommendation: "Confirm that the target is expected to have DNS records.".to_string(),
        }],
        Ok(addresses) => vec![ReconObservation {
            title: "DNS resolution available".to_string(),
            description: format!(
                "The host resolved through PolyGlid's DNS capability to {} address(es).",
                addresses.len()
            ),
            recommendation: "Use resolved addresses only for authorized follow-up checks."
                .to_string(),
        }],
        Err(message) => vec![ReconObservation {
            title: "DNS resolution unavailable".to_string(),
            description: format!("The DNS capability did not resolve {trimmed}: {message}"),
            recommendation: "Approve dns-resolve for this plugin and verify the target name."
                .to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_empty_target() {
        let observations = analyze_target(" ", Ok(Vec::new()));
        assert_eq!(observations[0].title, "Empty target");
    }

    #[test]
    fn reports_loopback_target() {
        let observations = analyze_target("127.0.0.1", Ok(vec!["127.0.0.1".to_string()]));
        assert_eq!(observations[0].title, "Loopback target");
    }

    #[test]
    fn reports_dns_resolution() {
        let observations = analyze_target("example.com", Ok(vec!["93.184.216.34".to_string()]));
        assert_eq!(observations[0].title, "DNS resolution available");
    }

    #[test]
    fn reports_dns_denial() {
        let observations = analyze_target("example.com", Err("denied".to_string()));
        assert_eq!(observations[0].title, "DNS resolution unavailable");
    }

    #[test]
    fn sanitizes_report_filename_target() {
        assert_eq!(report_safe_target("local/host:443"), "local-host-443");
    }
}
