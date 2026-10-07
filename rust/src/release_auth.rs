use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::{DataPackageManifest, RUNTIME_API_VERSION};

pub const SIGNATURE_ENVELOPE_VERSION: u32 = 1;
pub const RELEASE_CATALOG_VERSION: u32 = 1;
pub const PACKAGE_MANIFEST_DOMAIN: &[u8] = b"CRL-PACKAGE-MANIFEST-V1\0";
pub const RELEASE_CATALOG_DOMAIN: &[u8] = b"CRL-RELEASE-CATALOG-V1\0";

#[derive(Debug)]
pub enum ReleaseAuthError {
    Json(serde_json::Error),
    Invalid(String),
}

impl From<serde_json::Error> for ReleaseAuthError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub type ReleaseAuthResult<T> = std::result::Result<T, ReleaseAuthError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedKey {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_base64: String,
}

impl TrustedKey {
    pub fn from_verifying_key(key: &VerifyingKey) -> Self {
        let bytes = key.to_bytes();
        Self {
            key_id: key_id_for_public_key(&bytes),
            algorithm: "ed25519".into(),
            public_key_base64: STANDARD.encode(bytes),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignatureEnvelope {
    pub signature_version: u32,
    pub algorithm: String,
    pub key_id: String,
    pub signature_base64: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignedPayloadKind {
    PackageManifest,
    ReleaseCatalog,
}

impl SignedPayloadKind {
    fn domain(self) -> &'static [u8] {
        match self {
            Self::PackageManifest => PACKAGE_MANIFEST_DOMAIN,
            Self::ReleaseCatalog => RELEASE_CATALOG_DOMAIN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrustedKeySet {
    keys: HashMap<String, VerifyingKey>,
}

impl TrustedKeySet {
    pub fn from_entries(entries: &[TrustedKey]) -> ReleaseAuthResult<Self> {
        let mut keys = HashMap::new();
        for entry in entries {
            if entry.algorithm != "ed25519" {
                return Err(ReleaseAuthError::Invalid(format!(
                    "Unsupported trusted-key algorithm for {}",
                    entry.key_id
                )));
            }
            let decoded = STANDARD
                .decode(&entry.public_key_base64)
                .map_err(|_| ReleaseAuthError::Invalid("Invalid trusted-key base64".into()))?;
            let bytes: [u8; 32] = decoded.try_into().map_err(|_| {
                ReleaseAuthError::Invalid("Ed25519 public key must be 32 bytes".into())
            })?;
            let expected_id = key_id_for_public_key(&bytes);
            if entry.key_id != expected_id {
                return Err(ReleaseAuthError::Invalid(format!(
                    "Trusted key ID does not match its public key: {}",
                    entry.key_id
                )));
            }
            let key = VerifyingKey::from_bytes(&bytes)
                .map_err(|_| ReleaseAuthError::Invalid("Invalid Ed25519 public key".into()))?;
            if keys.insert(entry.key_id.clone(), key).is_some() {
                return Err(ReleaseAuthError::Invalid(format!(
                    "Duplicate trusted key ID: {}",
                    entry.key_id
                )));
            }
        }
        if keys.is_empty() {
            return Err(ReleaseAuthError::Invalid(
                "Trusted key set must not be empty".into(),
            ));
        }
        Ok(Self { keys })
    }

    pub fn contains(&self, key_id: &str) -> bool {
        self.keys.contains_key(key_id)
    }

    fn get(&self, key_id: &str) -> Option<&VerifyingKey> {
        self.keys.get(key_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseCatalogPackage {
    pub package_id: String,
    pub version: String,
    pub pack_type: String,
    pub min_runtime_api: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseCatalog {
    pub catalog_version: u32,
    pub sequence: u64,
    pub generated_at_unix: i64,
    pub expires_at_unix: i64,
    pub packages: Vec<ReleaseCatalogPackage>,
}

pub fn key_id_for_public_key(public_key: &[u8; 32]) -> String {
    format!("ed25519-sha256:{:x}", Sha256::digest(public_key))
}

pub fn sign_detached(
    kind: SignedPayloadKind,
    payload: &[u8],
    signing_key: &SigningKey,
) -> SignatureEnvelope {
    let message = domain_separated(kind, payload);
    let signature: Signature = signing_key.sign(&message);
    let verifying_key = signing_key.verifying_key();
    SignatureEnvelope {
        signature_version: SIGNATURE_ENVELOPE_VERSION,
        algorithm: "ed25519".into(),
        key_id: key_id_for_public_key(&verifying_key.to_bytes()),
        signature_base64: STANDARD.encode(signature.to_bytes()),
    }
}

pub fn verify_detached(
    kind: SignedPayloadKind,
    payload: &[u8],
    envelope: &SignatureEnvelope,
    trusted_keys: &TrustedKeySet,
) -> ReleaseAuthResult<String> {
    if envelope.signature_version != SIGNATURE_ENVELOPE_VERSION {
        return Err(ReleaseAuthError::Invalid(format!(
            "Unsupported signature envelope version: {}",
            envelope.signature_version
        )));
    }
    if envelope.algorithm != "ed25519" {
        return Err(ReleaseAuthError::Invalid(
            "Unsupported signature algorithm".into(),
        ));
    }
    let key = trusted_keys
        .get(&envelope.key_id)
        .ok_or_else(|| ReleaseAuthError::Invalid("Signature uses an unknown key ID".into()))?;
    let signature_bytes = STANDARD
        .decode(&envelope.signature_base64)
        .map_err(|_| ReleaseAuthError::Invalid("Invalid signature base64".into()))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| ReleaseAuthError::Invalid("Ed25519 signature must be 64 bytes".into()))?;
    let message = domain_separated(kind, payload);
    key.verify_strict(&message, &signature)
        .map_err(|_| ReleaseAuthError::Invalid("Ed25519 signature verification failed".into()))?;
    Ok(envelope.key_id.clone())
}

pub fn verify_package_manifest_signature(
    package_manifest_bytes: &[u8],
    envelope: &SignatureEnvelope,
    trusted_keys: &TrustedKeySet,
) -> ReleaseAuthResult<String> {
    verify_detached(
        SignedPayloadKind::PackageManifest,
        package_manifest_bytes,
        envelope,
        trusted_keys,
    )
}

pub fn verify_release_catalog(
    catalog_bytes: &[u8],
    envelope: &SignatureEnvelope,
    trusted_keys: &TrustedKeySet,
    now_unix: i64,
    highest_seen_sequence: u64,
) -> ReleaseAuthResult<ReleaseCatalog> {
    verify_detached(
        SignedPayloadKind::ReleaseCatalog,
        catalog_bytes,
        envelope,
        trusted_keys,
    )?;
    let catalog: ReleaseCatalog = serde_json::from_slice(catalog_bytes)?;
    validate_catalog(&catalog, now_unix, highest_seen_sequence)?;
    Ok(catalog)
}

pub fn verify_catalog_package_binding(
    entry: &ReleaseCatalogPackage,
    package_manifest_bytes: &[u8],
) -> ReleaseAuthResult<DataPackageManifest> {
    let actual_hash = format!("{:x}", Sha256::digest(package_manifest_bytes));
    if actual_hash != entry.manifest_sha256.to_ascii_lowercase() {
        return Err(ReleaseAuthError::Invalid(
            "Package manifest SHA-256 does not match signed catalog".into(),
        ));
    }
    let manifest: DataPackageManifest = serde_json::from_slice(package_manifest_bytes)?;
    if manifest.package_id != entry.package_id
        || manifest.version != entry.version
        || manifest.pack_type != entry.pack_type
        || manifest.min_runtime_api != entry.min_runtime_api
    {
        return Err(ReleaseAuthError::Invalid(
            "Package manifest identity does not match signed catalog entry".into(),
        ));
    }
    Ok(manifest)
}

fn validate_catalog(
    catalog: &ReleaseCatalog,
    now_unix: i64,
    highest_seen_sequence: u64,
) -> ReleaseAuthResult<()> {
    if catalog.catalog_version != RELEASE_CATALOG_VERSION {
        return Err(ReleaseAuthError::Invalid(format!(
            "Unsupported release catalog version: {}",
            catalog.catalog_version
        )));
    }
    if catalog.sequence == 0 {
        return Err(ReleaseAuthError::Invalid(
            "Release catalog sequence must be positive".into(),
        ));
    }
    if catalog.sequence < highest_seen_sequence {
        return Err(ReleaseAuthError::Invalid(format!(
            "Release catalog rollback detected: received {}, highest trusted {}",
            catalog.sequence, highest_seen_sequence
        )));
    }
    if catalog.expires_at_unix <= now_unix {
        return Err(ReleaseAuthError::Invalid(
            "Release catalog has expired".into(),
        ));
    }
    if catalog.generated_at_unix > catalog.expires_at_unix {
        return Err(ReleaseAuthError::Invalid(
            "Release catalog expiry precedes its generation time".into(),
        ));
    }

    let mut identities = HashSet::new();
    for package in &catalog.packages {
        validate_token(&package.package_id, "package_id")?;
        validate_token(&package.version, "version")?;
        if !matches!(
            package.pack_type.as_str(),
            "core" | "attribution" | "sharealike"
        ) {
            return Err(ReleaseAuthError::Invalid(
                "Release catalog contains invalid pack_type".into(),
            ));
        }
        package
            .min_runtime_api
            .parse::<u64>()
            .map_err(|_| ReleaseAuthError::Invalid("min_runtime_api must be numeric".into()))?;
        validate_sha256(&package.manifest_sha256)?;
        if !identities.insert((package.package_id.clone(), package.version.clone())) {
            return Err(ReleaseAuthError::Invalid(format!(
                "Duplicate package/version in release catalog: {} {}",
                package.package_id, package.version
            )));
        }
    }
    Ok(())
}

pub fn package_is_runtime_compatible(package: &ReleaseCatalogPackage) -> bool {
    match (
        package.min_runtime_api.parse::<u64>(),
        RUNTIME_API_VERSION.parse::<u64>(),
    ) {
        (Ok(required), Ok(current)) => required <= current,
        _ => false,
    }
}

fn validate_token(value: &str, field: &str) -> ReleaseAuthResult<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(ReleaseAuthError::Invalid(format!("Unsafe {field}")));
    }
    Ok(())
}

fn validate_sha256(value: &str) -> ReleaseAuthResult<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 64 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return Err(ReleaseAuthError::Invalid(
            "SHA-256 value must contain exactly 64 hexadecimal characters".into(),
        ));
    }
    Ok(())
}

fn domain_separated(kind: SignedPayloadKind, payload: &[u8]) -> Vec<u8> {
    let domain = kind.domain();
    let mut message = Vec::with_capacity(domain.len() + payload.len());
    message.extend_from_slice(domain);
    message.extend_from_slice(payload);
    message
}
