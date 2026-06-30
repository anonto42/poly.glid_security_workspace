//! Harmless first-party demo plugin logic.

wit_bindgen::generate!({
    world: "security-tool",
    path: "../../wit",
});

use crate::polyglid::engine::{
    dns,
    types::{Issue, Severity},
};

struct ReconProbe;

impl Guest for ReconProbe {
    fn execute(target: String) -> Result<PluginReport, String> {
        let issues = analyze_target(&target, resolve_target(&target))
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

fn resolve_target(target: &str) -> Result<Vec<String>, String> {
    dns::resolve(target)
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
}
