// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! HMAC-like asset pack signing using SHA-256 double-hash construction.
//!
//! Signing scheme: `SHA256(key || SHA256(message))` — length-extension resistant.

#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

// ── Public structs ────────────────────────────────────────────────────────────

/// Signature metadata for a signed pack.
pub struct PackSignature {
    /// Format version — currently 1.
    pub version: u32,
    /// Algorithm identifier, e.g. "sha256-chain".
    pub algorithm: String,
    /// Raw 32-byte signature.
    pub signature: Vec<u8>,
    /// Unix timestamp in seconds (0 for deterministic tests).
    pub timestamp: u64,
    /// Identifies who signed the pack.
    pub signer_id: String,
}

/// A manifest hash paired with its signature.
pub struct SignedPack {
    /// SHA-256 of sorted manifest content.
    pub manifest_hash: Vec<u8>,
    /// The signature covering `manifest_hash`.
    pub signature: PackSignature,
}

// ── Core crypto primitive ─────────────────────────────────────────────────────

/// `SHA256(key || SHA256(message))` — length-extension resistant double-hash.
pub fn double_hash_sign(key: &[u8], message: &[u8]) -> Vec<u8> {
    // Inner hash: SHA256(message)
    let mut inner = Sha256::new();
    inner.update(message);
    let inner_hash = inner.finalize();

    // Outer hash: SHA256(key || inner_hash)
    let mut outer = Sha256::new();
    outer.update(key);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

// ── Manifest hash ─────────────────────────────────────────────────────────────

/// Compute SHA-256 over sorted `"path:sha256hex"` lines for all files in `dir`.
pub fn pack_manifest_hash(dir: &Path) -> Result<Vec<u8>> {
    let mut entries: Vec<(String, String)> = Vec::new();
    collect_manifest_entries(dir, dir, &mut entries)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel_path, sha_hex) in &entries {
        let line = format!("{}:{}\n", rel_path, sha_hex);
        hasher.update(line.as_bytes());
    }
    Ok(hasher.finalize().to_vec())
}

fn collect_manifest_entries(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    for entry in
        std::fs::read_dir(current).with_context(|| format!("reading dir {}", current.display()))?
    {
        let entry = entry.with_context(|| "dir entry error")?;
        let path = entry.path();
        if path.is_dir() {
            collect_manifest_entries(root, &path, out)?;
        } else {
            let data =
                std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let sha_hex = sha256_hex(&data);
            let rel = path
                .strip_prefix(root)
                .with_context(|| "strip prefix")?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, sha_hex));
        }
    }
    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

// ── Sign / verify ─────────────────────────────────────────────────────────────

/// Sign all files in `dir` and return a `SignedPack`.
pub fn sign_pack_dir(dir: &Path, key: &[u8], signer_id: &str) -> Result<SignedPack> {
    let manifest_hash = pack_manifest_hash(dir)?;
    let signature_bytes = double_hash_sign(key, &manifest_hash);

    Ok(SignedPack {
        manifest_hash: manifest_hash.clone(),
        signature: PackSignature {
            version: 1,
            algorithm: "sha256-chain".to_string(),
            signature: signature_bytes,
            timestamp: 0,
            signer_id: signer_id.to_string(),
        },
    })
}

/// Verify a `SignedPack` against the current state of `dir` using `key`.
pub fn verify_pack_signature(dir: &Path, signed: &SignedPack, key: &[u8]) -> bool {
    let current_hash = match pack_manifest_hash(dir) {
        Ok(h) => h,
        Err(_) => return false,
    };
    if current_hash != signed.manifest_hash {
        return false;
    }
    let expected_sig = double_hash_sign(key, &current_hash);
    expected_sig == signed.signature.signature
}

// ── Hex encoding ──────────────────────────────────────────────────────────────

/// Hex-encode the signature bytes.
pub fn signature_to_hex(sig: &PackSignature) -> String {
    hex::encode(&sig.signature)
}

/// Decode a hex signature string back into a `PackSignature`.
pub fn signature_from_hex(
    hex_str: &str,
    version: u32,
    algorithm: &str,
    timestamp: u64,
    signer_id: &str,
) -> Result<PackSignature> {
    let bytes = hex::decode(hex_str).with_context(|| "hex decode failed")?;
    if bytes.len() != 32 {
        bail!("signature must be exactly 32 bytes, got {}", bytes.len());
    }
    Ok(PackSignature {
        version,
        algorithm: algorithm.to_string(),
        signature: bytes,
        timestamp,
        signer_id: signer_id.to_string(),
    })
}

// ── File I/O ──────────────────────────────────────────────────────────────────

/// Write a `SignedPack` to a text file in simple key=value format.
pub fn write_signature_file(signed: &SignedPack, path: &Path) -> Result<()> {
    let sig = &signed.signature;
    let content = format!(
        "version={}\nalgorithm={}\ntimestamp={}\nsigner_id={}\nmanifest_hash={}\nsignature={}\n",
        sig.version,
        sig.algorithm,
        sig.timestamp,
        sig.signer_id,
        hex::encode(&signed.manifest_hash),
        signature_to_hex(sig),
    );
    std::fs::write(path, content)
        .with_context(|| format!("writing signature file {}", path.display()))?;
    Ok(())
}

/// Read a `SignedPack` from a text file written by `write_signature_file`.
pub fn read_signature_file(path: &Path) -> Result<SignedPack> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading signature file {}", path.display()))?;

    let mut version: Option<u32> = None;
    let mut algorithm: Option<String> = None;
    let mut timestamp: Option<u64> = None;
    let mut signer_id: Option<String> = None;
    let mut manifest_hash_hex: Option<String> = None;
    let mut signature_hex: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line
            .split_once('=')
            .with_context(|| format!("malformed line: {}", line))?;
        match k {
            "version" => version = Some(v.parse().with_context(|| "parsing version")?),
            "algorithm" => algorithm = Some(v.to_string()),
            "timestamp" => timestamp = Some(v.parse().with_context(|| "parsing timestamp")?),
            "signer_id" => signer_id = Some(v.to_string()),
            "manifest_hash" => manifest_hash_hex = Some(v.to_string()),
            "signature" => signature_hex = Some(v.to_string()),
            _ => bail!("unknown key: {}", k),
        }
    }

    let version = version.with_context(|| "missing version")?;
    let algorithm = algorithm.with_context(|| "missing algorithm")?;
    let timestamp = timestamp.with_context(|| "missing timestamp")?;
    let signer_id = signer_id.with_context(|| "missing signer_id")?;
    let manifest_hash_hex = manifest_hash_hex.with_context(|| "missing manifest_hash")?;
    let signature_hex = signature_hex.with_context(|| "missing signature")?;

    let manifest_hash =
        hex::decode(&manifest_hash_hex).with_context(|| "hex decode manifest_hash")?;
    let signature_bytes = hex::decode(&signature_hex).with_context(|| "hex decode signature")?;

    Ok(SignedPack {
        manifest_hash,
        signature: PackSignature {
            version,
            algorithm,
            signature: signature_bytes,
            timestamp,
            signer_id,
        },
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tempdir(suffix: &str) -> std::path::PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should succeed")
            .subsec_nanos();
        let path =
            std::path::PathBuf::from(format!("/tmp/oxihuman_pack_sign_{}_{}", suffix, nanos));
        std::fs::create_dir_all(&path).expect("should succeed");
        path
    }

    fn write_file(dir: &std::path::Path, name: &str, data: &[u8]) {
        let mut f = std::fs::File::create(dir.join(name)).expect("should succeed");
        f.write_all(data).expect("should succeed");
    }

    // 1. double_hash_sign is deterministic
    #[test]
    fn double_hash_sign_is_deterministic() {
        let key = b"secret-key";
        let msg = b"hello world";
        let a = double_hash_sign(key, msg);
        let b = double_hash_sign(key, msg);
        assert_eq!(a, b);
    }

    // 2. double_hash_sign produces 32 bytes
    #[test]
    fn double_hash_sign_is_32_bytes() {
        let sig = double_hash_sign(b"key", b"msg");
        assert_eq!(sig.len(), 32);
    }

    // 3. Different keys produce different signatures
    #[test]
    fn different_keys_produce_different_sigs() {
        let msg = b"same message";
        let s1 = double_hash_sign(b"key-one", msg);
        let s2 = double_hash_sign(b"key-two", msg);
        assert_ne!(s1, s2);
    }

    // 4. Different messages produce different signatures
    #[test]
    fn different_messages_produce_different_sigs() {
        let key = b"same-key";
        let s1 = double_hash_sign(key, b"message-a");
        let s2 = double_hash_sign(key, b"message-b");
        assert_ne!(s1, s2);
    }

    // 5. Empty message is handled
    #[test]
    fn empty_message_does_not_panic() {
        let sig = double_hash_sign(b"key", b"");
        assert_eq!(sig.len(), 32);
    }

    // 6. Empty key is handled
    #[test]
    fn empty_key_does_not_panic() {
        let sig = double_hash_sign(b"", b"message");
        assert_eq!(sig.len(), 32);
    }

    // 7. signature_to_hex / signature_from_hex round-trip
    #[test]
    fn signature_hex_roundtrip() {
        let raw = double_hash_sign(b"k", b"m");
        let sig = PackSignature {
            version: 1,
            algorithm: "sha256-chain".to_string(),
            signature: raw.clone(),
            timestamp: 42,
            signer_id: "tester".to_string(),
        };
        let hex_str = signature_to_hex(&sig);
        let recovered = signature_from_hex(
            &hex_str,
            sig.version,
            &sig.algorithm,
            sig.timestamp,
            &sig.signer_id,
        )
        .expect("should succeed");
        assert_eq!(recovered.signature, raw);
        assert_eq!(recovered.version, 1);
        assert_eq!(recovered.algorithm, "sha256-chain");
        assert_eq!(recovered.timestamp, 42);
        assert_eq!(recovered.signer_id, "tester");
    }

    // 8. signature_from_hex rejects bad length
    #[test]
    fn signature_from_hex_rejects_wrong_length() {
        let bad_hex = hex::encode(b"tooshort");
        let result = signature_from_hex(&bad_hex, 1, "sha256-chain", 0, "tester");
        assert!(result.is_err());
    }

    // 9. write_signature_file / read_signature_file round-trip
    #[test]
    fn write_read_signature_file_roundtrip() {
        let tmp = tempdir("roundtrip");
        let sig_path = tmp.join("sig.txt");

        // Build a signed pack with known data
        let raw_sig = double_hash_sign(b"roundtrip-key", b"roundtrip-data");
        let signed = SignedPack {
            manifest_hash: double_hash_sign(b"", b"manifest"),
            signature: PackSignature {
                version: 1,
                algorithm: "sha256-chain".to_string(),
                signature: raw_sig.clone(),
                timestamp: 0,
                signer_id: "ci-bot".to_string(),
            },
        };

        write_signature_file(&signed, &sig_path).expect("should succeed");
        let recovered = read_signature_file(&sig_path).expect("should succeed");

        assert_eq!(recovered.manifest_hash, signed.manifest_hash);
        assert_eq!(recovered.signature.signature, raw_sig);
        assert_eq!(recovered.signature.version, 1);
        assert_eq!(recovered.signature.signer_id, "ci-bot");
    }

    // 10. verify_pack_signature succeeds on consistent data
    #[test]
    fn verify_pack_signature_succeeds_on_valid_data() {
        let tmp = tempdir("verify_ok");
        write_file(&tmp, "a.bin", b"alpha");
        write_file(&tmp, "b.bin", b"beta");

        let key = b"correct-key";
        let signed = sign_pack_dir(&tmp, key, "test-signer").expect("should succeed");
        assert!(verify_pack_signature(&tmp, &signed, key));
    }

    // 11. Wrong key fails verification
    #[test]
    fn verify_pack_signature_fails_wrong_key() {
        let tmp = tempdir("verify_wrong_key");
        write_file(&tmp, "x.bin", b"data");

        let signed = sign_pack_dir(&tmp, b"correct-key", "signer").expect("should succeed");
        assert!(!verify_pack_signature(&tmp, &signed, b"wrong-key"));
    }

    // 12. Tampered file fails verification
    #[test]
    fn verify_pack_signature_fails_tampered_file() {
        let tmp = tempdir("verify_tampered");
        write_file(&tmp, "file.bin", b"original");

        let key = b"tamper-key";
        let signed = sign_pack_dir(&tmp, key, "signer").expect("should succeed");

        // Tamper after signing
        write_file(&tmp, "file.bin", b"tampered!");

        assert!(!verify_pack_signature(&tmp, &signed, key));
    }

    // 13. pack_manifest_hash is stable
    #[test]
    fn pack_manifest_hash_is_stable() {
        let tmp = tempdir("manifest_stable");
        write_file(&tmp, "c.bin", b"gamma");
        write_file(&tmp, "a.bin", b"alpha");
        write_file(&tmp, "b.bin", b"beta");

        let h1 = pack_manifest_hash(&tmp).expect("should succeed");
        let h2 = pack_manifest_hash(&tmp).expect("should succeed");
        assert_eq!(h1, h2);
    }
}
