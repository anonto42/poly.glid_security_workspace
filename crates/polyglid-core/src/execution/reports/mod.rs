pub mod json;
pub mod markdown;
pub mod html;
pub mod sarif;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportMetadata {
    pub polyglid_version: String,
    pub plugin_id: String,
    pub plugin_version: String,
    pub target: String,
    pub timestamp: u64,
    pub security_profile: String,
    pub execution_duration_ms: u64,
    pub report_format_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedReport {
    pub metadata: ReportMetadata,
    pub report: polyglid_plugin_api::PluginReport,
}
