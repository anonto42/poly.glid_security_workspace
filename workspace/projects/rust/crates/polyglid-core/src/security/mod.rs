pub mod verifier;
pub mod trust_store;
pub mod permission_engine;
pub mod audit;
pub mod profiles;
pub mod publisher;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SignatureStatus {
    Verified,
    Invalid,
    Missing,
    UnknownPublisher,
    Revoked,
}

impl std::fmt::Display for SignatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Verified => "Verified",
            Self::Invalid => "Invalid",
            Self::Missing => "Missing",
            Self::UnknownPublisher => "UnknownPublisher",
            Self::Revoked => "Revoked",
        };
        f.write_str(s)
    }
}
