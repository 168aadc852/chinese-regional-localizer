use chinese_regional_localizer::{validate_package_dir, PackageStore};
use rusqlite::Connection;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn sha256(path: &Path) -> String {
    let bytes = fs::read(path).expect("read file");
    format!("{:x}", Sha256::digest(bytes))
}

fn create_shared_db(path: &Path, pack: &str) {
    let conn = Connection::open(path).expect("create sqlite");
    conn.execute_batch(
        "CREATE TABLE concepts(id INTEGER);\n         CREATE TABLE localized_names(id INTEGER);\n         CREATE TABLE term_rules(id INTEGER);\n         CREATE TABLE source_versions(id INTEGER);\n         CREATE TABLE build_metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )
    .expect("schema");
    conn.execute(
        "INSERT INTO build_metadata(key, value) VALUES ('pack_type', ?1)",
        [pack],
    )
    .expect("pack metadata");
}

fn create_package(root: &Path, package_id: &str, version: &str, pack: &str) -> PathBuf {
    let dir = root.join(format!("{package_id}-{version}"));
    fs::create_dir_all(&dir).expect("package dir");
    let db = dir.join("regional.sqlite");
    create_shared_db(&db, pack);
    let manifest = json!({
        "manifest_version": 1,
        "package_id": package_id,
        "version": version,
        "pack_type": pack,
        "min_runtime_api": "1",
        "created_at": "2026-10-07T00:00:00Z",
        "database": {
            "file": "regional.sqlite",
            "size_bytes": fs::metadata(&db).expect("metadata").len(),
            "sha256": sha256(&db)
        },
        "sources": [{
            "source_id": "opencc",
            "resource_key": "STCharacters.txt",
            "version_label": "test",
            "revision_id": "test",
            "checksum_sha256": null
        }]
    });
    fs::write(
        dir.join("package.json"),
        serde_json::to_vec_pretty(&manifest).expect("manifest json"),
    )
    .expect("write manifest");
    dir
}

#[test]
fn valid_package_is_accepted() {
    let temp = TempDir::new().expect("tempdir");
    let package = create_package(temp.path(), "regional-core", "1.0.0", "core");
    let validated = validate_package_dir(&package).expect("valid package");
    assert_eq!(validated.manifest.version, "1.0.0");
    assert!(validated.database_path.ends_with("regional.sqlite"));
}

#[test]
fn checksum_mismatch_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let package = create_package(temp.path(), "regional-core", "1.0.0", "core");
    fs::write(package.join("regional.sqlite"), b"changed").expect("tamper");
    assert!(validate_package_dir(&package).is_err());
}

#[test]
fn traversal_filename_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let package = create_package(temp.path(), "regional-core", "1.0.0", "core");
    let manifest_path = package.join("package.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest")).expect("json");
    manifest["database"]["file"] = json!("../regional.sqlite");
    fs::write(
        manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("json"),
    )
    .expect("write");
    assert!(validate_package_dir(&package).is_err());
}

#[test]
fn manifest_database_pack_mismatch_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let package = create_package(temp.path(), "regional-core", "1.0.0", "core");
    let manifest_path = package.join("package.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest")).expect("json");
    manifest["pack_type"] = json!("attribution");
    fs::write(
        manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("json"),
    )
    .expect("write");
    assert!(validate_package_dir(&package).is_err());
}

#[test]
fn corrupt_sqlite_is_rejected_even_with_matching_checksum() {
    let temp = TempDir::new().expect("tempdir");
    let package = temp.path().join("corrupt");
    fs::create_dir_all(&package).expect("dir");
    let db = package.join("regional.sqlite");
    fs::write(&db, b"not sqlite").expect("write corrupt");
    let manifest = json!({
        "manifest_version": 1,
        "package_id": "regional-core",
        "version": "1.0.0",
        "pack_type": "core",
        "min_runtime_api": "1",
        "created_at": "2026-10-07T00:00:00Z",
        "database": {
            "file": "regional.sqlite",
            "size_bytes": fs::metadata(&db).expect("metadata").len(),
            "sha256": sha256(&db)
        },
        "sources": [{"source_id":"opencc","resource_key":"x","version_label":null,"revision_id":null,"checksum_sha256":null}]
    });
    fs::write(package.join("package.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(validate_package_dir(&package).is_err());
}

#[test]
fn install_activate_and_rollback_preserve_versions() {
    let temp = TempDir::new().expect("tempdir");
    let packages = temp.path().join("incoming");
    fs::create_dir_all(&packages).expect("incoming");
    let v1 = create_package(&packages, "regional-core", "1.0.0", "core");
    let v2 = create_package(&packages, "regional-core", "2.0.0", "core");
    let store = PackageStore::new(temp.path().join("store"));

    store.install(&v1).expect("install v1");
    store.install(&v2).expect("install v2");
    let state = store.state().expect("state");
    assert_eq!(state.current.as_ref().unwrap().version, "2.0.0");
    assert_eq!(state.previous.as_ref().unwrap().version, "1.0.0");

    let rolled_back = store.rollback().expect("rollback");
    assert!(rolled_back.to_string_lossy().contains("1.0.0"));
    let state = store.state().expect("state after rollback");
    assert_eq!(state.current.as_ref().unwrap().version, "1.0.0");
    assert_eq!(state.previous.as_ref().unwrap().version, "2.0.0");
}

#[test]
fn failed_install_does_not_replace_current_package() {
    let temp = TempDir::new().expect("tempdir");
    let packages = temp.path().join("incoming");
    fs::create_dir_all(&packages).expect("incoming");
    let v1 = create_package(&packages, "regional-core", "1.0.0", "core");
    let bad = create_package(&packages, "regional-core", "2.0.0", "core");
    fs::write(bad.join("regional.sqlite"), b"tampered").expect("tamper");
    let store = PackageStore::new(temp.path().join("store"));

    store.install(&v1).expect("install v1");
    assert!(store.install(&bad).is_err());
    let state = store.state().expect("state");
    assert_eq!(state.current.as_ref().unwrap().version, "1.0.0");
    assert!(state.previous.is_none());
}
