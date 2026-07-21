use crate::execution::reports::ExportedReport;

pub fn export(report: &ExportedReport) -> Result<String, String> {
    let mut rules = Vec::new();
    let mut results = Vec::new();

    for (index, issue) in report.report.issues.iter().enumerate() {
        let rule_id = format!("PG{:04}", index + 1);

        let sarif_severity = match format!("{:?}", issue.severity).to_lowercase().as_str() {
            "critical" | "high" => "error",
            "medium" => "warning",
            _ => "note",
        };

        rules.push(serde_json::json!({
            "id": rule_id,
            "shortDescription": {
                "text": issue.title
            },
            "helpUri": "https://github.com/anonto42/poly.glid_security_workspace"
        }));

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "message": {
                "text": format!("{}\nRecommendation: {}", issue.description, issue.recommendation)
            },
            "level": sarif_severity,
            "locations": [
                {
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": report.metadata.target
                        }
                    }
                }
            ]
        }));
    }

    let sarif = serde_json::json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "PolyGlid",
                        "semanticVersion": report.metadata.polyglid_version,
                        "rules": rules
                    }
                },
                "results": results
            }
        ]
    });

    serde_json::to_string_pretty(&sarif)
        .map_err(|err| format!("failed to serialize report to SARIF: {err}"))
}
