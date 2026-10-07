use chinese_regional_localizer::{
    sign_detached, AuthenticatedCatalog, DataPackageManifest, PackageDatabase, PackageSource,
    PackageStore, ReleaseCatalog, ReleaseCatalogPackage, SignatureEnvelope, SignedPayloadKind,
    TrustedKey, TrustedKeySet, UpdateClient, UpdateConfig, UpdateError, UpdateTransport,
};
use ed25519_dalek::SigningKey;
use reqwest::Url;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust crate inside repo")
        .to_path_buf()
}

#[derive(Clone, Default)]
struct FakeTransport {
    responses: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl FakeTransport {
    fn insert(&self, url: &str, bytes: Vec<u8>) {
        self.responses.lock().unwrap().insert(url.into(), bytes);
    }
}

impl UpdateTransport for FakeTransport {
    fn get(&self, url: &Url, max_bytes: usize) -> Result<Vec<u8>, UpdateError> {
        let bytes = self
            .responses
            .lock()
            .unwrap()
            .get(url.as_str())
            .cloned()
            .ok_or_else(|| UpdateError::Invalid(format!("missing fake URL: {url}")))?;
        if bytes.len() > max_bytes {
            return Err(UpdateError::Invalid("fake response exceeds size limit".into()));
        }
        Ok(bytes)
    }
}

fn signed_bytes(
    kind: SignedPayloadKind,
    payload: &[u8],
    key: &SigningKey,
) -> Vec<u8> {
    serde_json::to_vec(&sign_detached(kind, payload, key)).unwrap()
}

fn trusted(key: &SigningKey) -> TrustedKeySet {
    TrustedKeySet::from_entries(&[TrustedKey::from_verifying_key(&key.verifying_key())]).unwrap()
}

fn demo_database_bytes() -> Vec<u8> {
    fs::read(repo_root().join("build/regional-demo.sqlite")).expect("demo DB built before cargo test")
}

fn manifest(version: &str, db: &[u8]) -> (Vec<u8>, DataPackageManifest) {
    let manifest = DataPackageManifest {
        manifest_version: 1,
        package_id: "core-regional".into(),
        version: version.into(),
        pack_type: "core".into(),
        min_runtime_api: "1".into(),
        created_at: "2026-10-07T00:00:00Z".into(),
        database: PackageDatabase {
            file: "regional.sqlite".into(),
            size_bytes: db.len() as u64,
            sha256: format!("{:x}", Sha256::digest(db)),
        },
        sources: vec![PackageSource {
            source_id: "opencc".into(),
            resource_key: "fixture".into(),
            version_label: Some("fixture".into()),
            revision_id: None,
            checksum_sha256: None,
        }],
    };
    let bytes = serde_json::to_vec(&manifest).unwrap();
    (bytes, manifest)
}

fn catalog(version: &str, manifest_bytes: &[u8], sequence: u64) -> Vec<u8> {
    serde_json::to_vec(&ReleaseCatalog {
        catalog_version: 1,
        sequence,
        generated_at_unix: 1_000,
        expires_at_unix: 10_000,
        packages: vec![ReleaseCatalogPackage {
            package_id: "core-regional".into(),
            version: version.into(),
            pack_type: "core".into(),
            min_runtime_api: "1".into(),
            manifest_sha256: format!("{:x}", Sha256::digest(manifest_bytes)),
        }],
    })
    .unwrap()
}

fn seed_release(
    transport: &FakeTransport,
    key: &SigningKey,
    version: &str,
    sequence: u64,
    served_database: &[u8],
    signed_database: &[u8],
) {
    let base = "https://updates.example.test/";
    let (manifest_bytes, _) = manifest(version, signed_database);
    let catalog_bytes = catalog(version, &manifest_bytes, sequence);
    transport.insert(&format!("{base}release-catalog.json"), catalog_bytes.clone());
    transport.insert(
        &format!("{base}release-catalog.json.sig"),
        signed_bytes(SignedPayloadKind::ReleaseCatalog, &catalog_bytes, key),
    );
    let prefix = format!("{base}packages/core-regional/{version}/");
    transport.insert(&(prefix.clone() + "package.json"), manifest_bytes.clone());
    transport.insert(
        &(prefix.clone() + "package.json.sig"),
        signed_bytes(SignedPayloadKind::PackageManifest, &manifest_bytes, key),
    );
    transport.insert(&(prefix + "regional.sqlite"), served_database.to_vec());
}

fn client(
    temp: &tempfile::TempDir,
    transport: FakeTransport,
    key: &SigningKey,
) -> UpdateClient<FakeTransport> {
    UpdateClient::new(
        transport,
        UpdateConfig::production("https://updates.example.test/").unwrap(),
        trusted(key),
        PackageStore::new(temp.path()),
    )
    .unwrap()
}

#[test]
fn production_base_url_requires_https_and_no_credentials() {
    assert!(UpdateConfig::production("http://updates.example.test/").is_err());
    assert!(UpdateConfig::production("https://user:pass@updates.example.test/").is_err());
    assert!(UpdateConfig::production("https://updates.example.test/root/").is_ok());
}

#[test]
fn authenticated_discovery_persists_highest_sequence_and_rejects_rollback() {
    let temp = tempfile::tempdir().unwrap();
    let transport = FakeTransport::default();
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let db = demo_database_bytes();
    seed_release(&transport, &key, "1.0.0", 5, &db, &db);
    let client = client(&temp, transport.clone(), &key);

    let catalog = client.discover(2_000).unwrap();
    assert_eq!(catalog.catalog().sequence, 5);
    assert_eq!(client.state().unwrap().highest_trusted_catalog_sequence, 5);

    seed_release(&transport, &key, "0.9.0", 4, &db, &db);
    assert!(client.discover(2_000).is_err());
    assert_eq!(client.state().unwrap().highest_trusted_catalog_sequence, 5);
}

#[test]
fn authenticated_package_installs_only_after_manifest_and_database_validation() {
    let temp = tempfile::tempdir().unwrap();
    let transport = FakeTransport::default();
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let db = demo_database_bytes();
    seed_release(&transport, &key, "1.0.0", 1, &db, &db);
    let client = client(&temp, transport, &key);

    let catalog: AuthenticatedCatalog = client.discover(2_000).unwrap();
    let installed = client
        .install_from_catalog(&catalog, "core-regional", "1.0.0")
        .unwrap();
    assert_eq!(installed.package_id, "core-regional");
    assert_eq!(installed.version, "1.0.0");
    assert_eq!(
        PackageStore::new(temp.path())
            .state()
            .unwrap()
            .current
            .unwrap()
            .version,
        "1.0.0"
    );
}

#[test]
fn corrupt_download_does_not_replace_active_package() {
    let temp = tempfile::tempdir().unwrap();
    let transport = FakeTransport::default();
    let key = SigningKey::from_bytes(&[11u8; 32]);
    let db = demo_database_bytes();
    seed_release(&transport, &key, "1.0.0", 1, &db, &db);
    let client = client(&temp, transport.clone(), &key);
    let first = client.discover(2_000).unwrap();
    client
        .install_from_catalog(&first, "core-regional", "1.0.0")
        .unwrap();

    let mut corrupt = db.clone();
    corrupt[0] ^= 0xff;
    seed_release(&transport, &key, "2.0.0", 2, &corrupt, &db);
    let second = client.discover(2_000).unwrap();
    assert!(client
        .install_from_catalog(&second, "core-regional", "2.0.0")
        .is_err());

    let state = PackageStore::new(temp.path()).state().unwrap();
    assert_eq!(state.current.unwrap().version, "1.0.0");
    assert!(state.previous.is_none());
}

#[test]
fn tampered_catalog_signature_is_rejected_without_advancing_sequence() {
    let temp = tempfile::tempdir().unwrap();
    let transport = FakeTransport::default();
    let key = SigningKey::from_bytes(&[13u8; 32]);
    let db = demo_database_bytes();
    seed_release(&transport, &key, "1.0.0", 3, &db, &db);
    let bad_signature = SignatureEnvelope {
        signature_version: 1,
        algorithm: "ed25519".into(),
        key_id: TrustedKey::from_verifying_key(&key.verifying_key()).key_id,
        signature_base64: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".into(),
    };
    transport.insert(
        "https://updates.example.test/release-catalog.json.sig",
        serde_json::to_vec(&bad_signature).unwrap(),
    );
    let client = client(&temp, transport, &key);
    assert!(client.discover(2_000).is_err());
    assert_eq!(client.state().unwrap().highest_trusted_catalog_sequence, 0);
}
