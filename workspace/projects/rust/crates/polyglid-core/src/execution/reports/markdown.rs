use crate::execution::reports::ExportedReport;

pub fn export(report: &ExportedReport) -> Result<String, String> {
    let mut md = String::new();
    md.push_str("# PolyGlid Scan Report\n\n");
    md.push_str("## Metadata\n\n");
    md.push_str("| Key | Value |\n");
    md.push_str("|---|---|\n");
    md.push_str(&format!("| **PolyGlid Version** | {} |\n", report.metadata.polyglid_version));
    md.push_str(&format!("| **Plugin ID** | {} |\n", report.metadata.plugin_id));
    md.push_str(&format!("| **Plugin Version** | {} |\n", report.metadata.plugin_version));
    md.push_str(&format!("| **Target** | {} |\n", report.metadata.target));
    md.push_str(&format!("| **Timestamp** | {} |\n", report.metadata.timestamp));
    md.push_str(&format!("| **Security Profile** | {} |\n", report.metadata.security_profile));
    md.push_str(&format!("| **Execution Duration** | {} ms |\n", report.metadata.execution_duration_ms));
    md.push_str(&format!("| **Report Format Version** | {} |\n\n", report.metadata.report_format_version));

    md.push_str("## Summary\n\n");
    md.push_str(&format!("{}\n\n", report.report.summary));

    md.push_str("## Observations / Issues\n\n");
    if report.report.issues.is_empty() {
        md.push_str("No issues identified during this scan.\n");
    } else {
        md.push_str("| Severity | Title | Description | Recommendation |\n");
        md.push_str("|---|---|---|---|\n");
        for issue in &report.report.issues {
            md.push_str(&format!(
                "| `{:?}` | {} | {} | {} |\n",
                issue.severity,
                issue.title,
                issue.description.replace("\n", " "),
                issue.recommendation.replace("\n", " ")
            ));
        }
    }

    Ok(md)
}
