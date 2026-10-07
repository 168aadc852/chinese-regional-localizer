use chinese_regional_localizer::{
    key_id_for_public_key, sign_detached, verify_catalog_package_binding, verify_detached,
    verify_package_manifest_signature, verify_release_catalog, DataPackageManifest,
    ReleaseCatalog, ReleaseCatalogPackage, SignatureEnvelope, SignedPayloadKind, TrustedKey,
    TrustedKeySet,
};
use ed25519_dalek::SigningKey;
use serde_json::json;
use sha2::{Digest, Sha256};

fn signing_key(seed_byte: u8) -> SigningKey {
    // Deterministic TEST-ONLY key material. Never use this helper for a real release key.
    SigningKey::from_bytes(&[seed_byte; 32])
}

fn trust_for(key: &SigningKey) -> TrustedKeySet {
    TrustedKeySet::from_entries(&[TrustedKey::from_verifying_key(&key.verifying_key())])
        .expect("valid trust set")
}

fn catalog_bytes(sequence: u64, expires_at_unix: i64) -> Vec<u8> {
    serde_json::to_vec(&ReleaseCatalog {
        catalog_version: 1,
        sequence,
        generated_at_unix: 1_000,
        expires_at_unix,
        packages: vec![ReleaseCatalogPackage {
            package_id: "regional-core".into(),
            version: "1.0.0".into(),
            pack_type: "core".into(),
            min_runtime_api: "1".into(),
            manifest_sha256: "a".repeat(64),
        }],
    })
    .expect("catalog json")
}

#[test]
fn valid_package_manifest_signature_is_accepted() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let payload = br#"{"manifest_version":1}"#;
    let signature = sign_detached(SignedPayloadKind::PackageManifest, payload, &key);
    let key_id = verify_package_manifest_signature(payload, &signature, &trust).expect("valid");
    assert_eq!(key_id, signature.key_id);
}

#[test]
fn tampered_payload_is_rejected() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let signature = sign_detached(SignedPayloadKind::PackageManifest, b"original", &key);
    assert!(verify_package_manifest_signature(b"changed", &signature, &trust).is_err());
}

#[test]
fn unknown_or_wrong_key_is_rejected() {
    let signer = signing_key(7);
    let other = signing_key(9);
    let trust = trust_for(&other);
    let signature = sign_detached(SignedPayloadKind::PackageManifest, b"payload", &signer);
    assert!(verify_package_manifest_signature(b"payload", &signature, &trust).is_err());
}

#[test]
fn domain_separation_blocks_signature_reuse() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let payload = b"same raw bytes";
    let signature = sign_detached(SignedPayloadKind::PackageManifest, payload, &key);
    assert!(verify_detached(
        SignedPayloadKind::ReleaseCatalog,
        payload,
        &signature,
        &trust
    )
    .is_err());
}

#[test]
fn expired_catalog_is_rejected_after_signature_verification() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let payload = catalog_bytes(4, 2_000);
    let signature = sign_detached(SignedPayloadKind::ReleaseCatalog, &payload, &key);
    assert!(verify_release_catalog(&payload, &signature, &trust, 2_000, 4).is_err());
}

#[test]
fn older_catalog_sequence_is_rejected_as_rollback() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let payload = catalog_bytes(4, 9_999);
    let signature = sign_detached(SignedPayloadKind::ReleaseCatalog, &payload, &key);
    assert!(verify_release_catalog(&payload, &signature, &trust, 2_000, 5).is_err());
}

#[test]
fn current_or_newer_catalog_sequence_is_accepted() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let payload = catalog_bytes(5, 9_999);
    let signature = sign_detached(SignedPayloadKind::ReleaseCatalog, &payload, &key);
    let catalog =
        verify_release_catalog(&payload, &signature, &trust, 2_000, 5).expect("valid catalog");
    assert_eq!(catalog.sequence, 5);
}

#[test]
fn catalog_entry_is_bound_to_exact_package_manifest_bytes() {
    let manifest = json!({
        "manifest_version": 1,
        "package_id": "regional-core",
        "version": "1.0.0",
        "pack_type": "core",
        "min_runtime_api": "1",
        "created_at": "2026-10-07T00:00:00Z",
        "database": {"file":"regional.sqlite","size_bytes":123,"sha256":"b".repeat(64)},
        "sources": [{"source_id":"opencc","resource_key":"opencc:STCharacters.txt","version_label":"test","revision_id":"test","checksum_sha256":null}]
    });
    let bytes = serde_json::to_vec(&manifest).expect("manifest bytes");
    let entry = ReleaseCatalogPackage {
        package_id: "regional-core".into(),
        version: "1.0.0".into(),
        pack_type: "core".into(),
        min_runtime_api: "1".into(),
        manifest_sha256: format!("{:x}", Sha256::digest(&bytes)),
    };
    let parsed: DataPackageManifest =
        verify_catalog_package_binding(&entry, &bytes).expect("binding valid");
    assert_eq!(parsed.package_id, "regional-core");

    let mut changed = bytes.clone();
    changed.push(b' ');
    assert!(verify_catalog_package_binding(&entry, &changed).is_err());
}

#[test]
fn trusted_key_id_must_match_pinned_public_key_bytes() {
    let key = signing_key(7);
    let mut entry = TrustedKey::from_verifying_key(&key.verifying_key());
    entry.key_id = key_id_for_public_key(&signing_key(8).verifying_key().to_bytes());
    assert!(TrustedKeySet::from_entries(&[entry]).is_err());
}

#[test]
fn malformed_signature_envelope_is_rejected() {
    let key = signing_key(7);
    let trust = trust_for(&key);
    let envelope = SignatureEnvelope {
        signature_version: 1,
        algorithm: "ed25519".into(),
        key_id: TrustedKey::from_verifying_key(&key.verifying_key()).key_id,
        signature_base64: "not-base64!".into(),
    };
    assert!(verify_package_manifest_signature(b"payload", &envelope, &trust).is_err());
}
