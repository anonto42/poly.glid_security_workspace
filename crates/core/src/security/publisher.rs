use sha2::{Digest, Sha256};

pub struct PublisherManager;

impl PublisherManager {
    pub fn compute_fingerprint(public_key_hex: &str) -> Result<String, String> {
        let bytes = hex::decode(public_key_hex)
            .map_err(|err| format!("invalid public key hex string: {err}"))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }
}
