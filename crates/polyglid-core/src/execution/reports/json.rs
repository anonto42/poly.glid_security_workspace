use crate::execution::reports::ExportedReport;

pub fn export(report: &ExportedReport) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map_err(|err| format!("failed to serialize report to JSON: {err}"))
}
