use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::fs;
use std::path::Path;

pub struct PluginVerifier;

impl PluginVerifier {
    pub fn verify(
        wasm_path: &Path,
        signature_bytes: &[u8],
        pub_key_bytes: &[u8]
    ) -> Result<(), String> {
        let verifying_key = VerifyingKey::from_bytes(
            pub_key_bytes.try_into().map_err(|_| "invalid public key length, expected 32 bytes")?
        ).map_err(|err| format!("invalid public key: {err}"))?;

        let signature = Signature::from_slice(signature_bytes)
            .map_err(|err| format!("invalid signature bytes: {err}"))?;

        let wasm_content = fs::read(wasm_path)
            .map_err(|err| format!("failed to read WASM file: {err}"))?;

        verifying_key.verify(&wasm_content, &signature)
            .map_err(|err| format!("cryptographic signature verification failed: {err}"))?;

        Ok(())
    }
}
