use polyglid_plugin_api::Capability;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityProfile {
    pub name: String,
    pub allowed_capabilities: Vec<Capability>,
    pub max_fuel: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub timeout_seconds: u64,
    pub require_signature: bool,
    pub require_trusted_publisher: bool,
}

impl SecurityProfile {
    pub fn strict() -> Self {
        Self {
            name: "Strict".to_string(),
            allowed_capabilities: vec![Capability::ReportWrite],
            max_fuel: Some(50_000),
            max_memory_bytes: Some(10 * 1024 * 1024),
            timeout_seconds: 5,
            require_signature: true,
            require_trusted_publisher: true,
        }
    }

    pub fn balanced() -> Self {
        Self {
            name: "Balanced".to_string(),
            allowed_capabilities: vec![
                Capability::DnsResolve,
                Capability::ReportWrite,
            ],
            max_fuel: Some(200_000),
            max_memory_bytes: Some(30 * 1024 * 1024),
            timeout_seconds: 15,
            require_signature: true,
            require_trusted_publisher: false,
        }
    }

    pub fn development() -> Self {
        Self {
            name: "Development".to_string(),
            allowed_capabilities: vec![
                Capability::DnsResolve,
                Capability::NetworkConnect,
                Capability::FilesystemRead,
                Capability::FilesystemWrite,
                Capability::ReportWrite,
            ],
            max_fuel: None,
            max_memory_bytes: None,
            timeout_seconds: 60,
            require_signature: false,
            require_trusted_publisher: false,
        }
    }
}
