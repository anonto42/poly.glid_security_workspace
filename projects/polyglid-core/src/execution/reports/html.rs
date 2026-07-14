use crate::execution::reports::ExportedReport;

pub fn export(report: &ExportedReport) -> Result<String, String> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>PolyGlid Scan Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif; background-color: #f6f8fa; margin: 0; padding: 40px; color: #24292e; }\n");
    html.push_str(".container { max-width: 900px; margin: 0 auto; background: #fff; padding: 30px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.08); }\n");
    html.push_str("h1 { border-bottom: 2px solid #e1e4e8; padding-bottom: 10px; margin-top: 0; }\n");
    html.push_str("table { width: 100%; border-collapse: collapse; margin: 20px 0; }\n");
    html.push_str("th, td { border: 1px solid #e1e4e8; padding: 12px; text-align: left; }\n");
    html.push_str("th { background-color: #f1f3f5; font-weight: bold; }\n");
    html.push_str(".severity-critical { color: #d73a49; font-weight: bold; }\n");
    html.push_str(".severity-high { color: #d73a49; }\n");
    html.push_str(".severity-medium { color: #e36209; }\n");
    html.push_str(".severity-low { color: #005cc5; }\n");
    html.push_str(".severity-info { color: #6a737d; }\n");
    html.push_str("</style>\n</head>\n<body>\n<div class=\"container\">\n");
    
    html.push_str("<h1>PolyGlid Scan Report</h1>\n");
    
    html.push_str("<h2>Metadata</h2>\n<table>\n");
    html.push_str(&format!("<tr><th>PolyGlid Version</th><td>{}</td></tr>\n", report.metadata.polyglid_version));
    html.push_str(&format!("<tr><th>Plugin ID</th><td>{}</td></tr>\n", report.metadata.plugin_id));
    html.push_str(&format!("<tr><th>Plugin Version</th><td>{}</td></tr>\n", report.metadata.plugin_version));
    html.push_str(&format!("<tr><th>Target</th><td>{}</td></tr>\n", report.metadata.target));
    html.push_str(&format!("<tr><th>Timestamp</th><td>{}</td></tr>\n", report.metadata.timestamp));
    html.push_str(&format!("<tr><th>Security Profile</th><td>{}</td></tr>\n", report.metadata.security_profile));
    html.push_str(&format!("<tr><th>Execution Duration</th><td>{} ms</td></tr>\n", report.metadata.execution_duration_ms));
    html.push_str(&format!("<tr><th>Report Format Version</th><td>{}</td></tr>\n", report.metadata.report_format_version));
    html.push_str("</table>\n");

    html.push_str("<h2>Summary</h2>\n");
    html.push_str(&format!("<p>{}</p>\n", report.report.summary));

    html.push_str("<h2>Observations / Issues</h2>\n");
    if report.report.issues.is_empty() {
        html.push_str("<p>No issues identified during this scan.</p>\n");
    } else {
        html.push_str("<table>\n<tr><th>Severity</th><th>Title</th><th>Description</th><th>Recommendation</th></tr>\n");
        for issue in &report.report.issues {
            let sev_class = match format!("{:?}", issue.severity).to_lowercase().as_str() {
                "critical" => "severity-critical",
                "high" => "severity-high",
                "medium" => "severity-medium",
                "low" => "severity-low",
                _ => "severity-info",
            };
            html.push_str(&format!(
                "<tr><td class=\"{}\">`{:?}`</td><td><b>{}</b></td><td>{}</td><td>{}</td></tr>\n",
                sev_class,
                issue.severity,
                issue.title,
                issue.description,
                issue.recommendation
            ));
        }
        html.push_str("</table>\n");
    }

    html.push_str("</div>\n</body>\n</html>\n");
    Ok(html)
}
