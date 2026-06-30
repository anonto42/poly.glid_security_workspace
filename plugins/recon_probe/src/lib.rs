//! Harmless first-party demo plugin logic.

wit_bindgen::generate!({
    world: "security-tool",
    path: "../../wit",
});

use crate::polyglid::engine::types::{Issue, Severity};

struct ReconProbe;

impl Guest for ReconProbe {
    fn execute(target: String) -> Result<PluginReport, String> {
        let issues = analyze_target(&target)
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
}

export!(ReconProbe);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconObservation {
    pub title: String,
    pub description: String,
    pub recommendation: String,
}

pub fn analyze_target(target: &str) -> Vec<ReconObservation> {
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

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_empty_target() {
        let observations = analyze_target(" ");
        assert_eq!(observations[0].title, "Empty target");
    }

    #[test]
    fn reports_loopback_target() {
        let observations = analyze_target("127.0.0.1");
        assert_eq!(observations[0].title, "Loopback target");
    }

    #[test]
    fn accepts_normal_target() {
        assert!(analyze_target("example.com").is_empty());
    }
}
