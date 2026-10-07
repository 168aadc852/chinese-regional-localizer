use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::RUNTIME_API_VERSION;

pub const PACKAGE_MANIFEST_VERSION: u32 = 1;
pub const PACKAGE_MANIFEST_FILE: &str = "package.json";

#[derive(Debug)]
pub enum PackageError {
    Io(io::Error),
    Json(serde_json::Error),
    Sql(rusqlite::Error),
    Invalid(String),
}

impl From<io::Error> for PackageError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<rusqlite::Error> for PackageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

pub type PackageResult<T> = std::result::Result<T, PackageError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageDatabase {
    pub file: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageSource {
    pub source_id: String,
    #[serde(default)]
    pub resource_key: String,
    pub version_label: Option<String>,
    pub revision_id: Option<String>,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageManifest {
    pub manifest_version: u32,
    pub package_id: String,
    pub version: String,
    pub pack_type: String,
    pub min_runtime_api: String,
    pub created_at: String,
    pub database: PackageDatabase,
    #[serde(default)]
    pub sources: Vec<PackageSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledPackageRef {
    pub package_id: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageStoreState {
    pub current: Option<InstalledPackageRef>,
    pub previous: Option<InstalledPackageRef>,
}

#[derive(Debug, Clone)]
pub struct ValidatedPackage {
    pub manifest: DataPackageManifest,
    pub database_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackageStore {
    root: PathBuf,
}

impl PackageStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn state(&self) -> PackageResult<PackageStoreState> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(PackageStoreState::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn install(&self, package_dir: impl AsRef<Path>) -> PackageResult<InstalledPackageRef> {
        let validated = validate_package_dir(package_dir.as_ref())?;
        fs::create_dir_all(self.packages_root())?;

        let package_ref = InstalledPackageRef {
            package_id: validated.manifest.package_id.clone(),
            version: validated.manifest.version.clone(),
        };
        let destination = self.package_dir(&package_ref);

        if destination.exists() {
            let installed = validate_package_dir(&destination)?;
            if installed.manifest.database.sha256 != validated.manifest.database.sha256 {
                return Err(PackageError::Invalid(
                    "An installed package with the same id/version has different contents".into(),
                ));
            }
        } else {
            let staging = self.root.join(format!(
                ".staging-{}-{}-{}",
                validated.manifest.package_id,
                validated.manifest.version,
                std::process::id()
            ));
            if staging.exists() {
                fs::remove_dir_all(&staging)?;
            }
            fs::create_dir_all(&staging)?;
            fs::copy(
                package_dir.as_ref().join(PACKAGE_MANIFEST_FILE),
                staging.join(PACKAGE_MANIFEST_FILE),
            )?;
            fs::copy(
                &validated.database_path,
                staging.join(&validated.manifest.database.file),
            )?;
            validate_package_dir(&staging)?;
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&staging, &destination)?;
        }

        self.activate(&package_ref)?;
        Ok(package_ref)
    }

    pub fn activate(&self, package_ref: &InstalledPackageRef) -> PackageResult<PathBuf> {
        let directory = self.package_dir(package_ref);
        let validated = validate_package_dir(&directory)?;
        let mut state = self.state()?;
        if state.current.as_ref() != Some(package_ref) {
            state.previous = state.current.take();
            state.current = Some(package_ref.clone());
            self.write_state(&state)?;
        }
        Ok(validated.database_path)
    }

    pub fn rollback(&self) -> PackageResult<PathBuf> {
        let mut state = self.state()?;
        let previous = state.previous.clone().ok_or_else(|| {
            PackageError::Invalid("No previous package is available for rollback".into())
        })?;
        let validated = validate_package_dir(&self.package_dir(&previous))?;
        let current = state.current.take();
        state.current = Some(previous);
        state.previous = current;
        self.write_state(&state)?;
        Ok(validated.database_path)
    }

    pub fn active_database_path(&self) -> PackageResult<Option<PathBuf>> {
        let state = self.state()?;
        let Some(current) = state.current else {
            return Ok(None);
        };
        let validated = validate_package_dir(&self.package_dir(&current))?;
        Ok(Some(validated.database_path))
    }

    fn packages_root(&self) -> PathBuf {
        self.root.join("packages")
    }

    fn package_dir(&self, package_ref: &InstalledPackageRef) -> PathBuf {
        self.packages_root()
            .join(&package_ref.package_id)
            .join(&package_ref.version)
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }

    fn write_state(&self, state: &PackageStoreState) -> PackageResult<()> {
        fs::create_dir_all(&self.root)?;
        let temp = self.root.join("state.json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(state)?)?;
        fs::rename(temp, self.state_path())?;
        Ok(())
    }
}

pub fn validate_package_dir(package_dir: &Path) -> PackageResult<ValidatedPackage> {
    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    let manifest: DataPackageManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    validate_manifest(&manifest)?;

    let database_path = package_dir.join(&manifest.database.file);
    if !database_path.is_file() {
        return Err(PackageError::Invalid(
            "Package database file is missing".into(),
        ));
    }
    let size = fs::metadata(&database_path)?.len();
    if size != manifest.database.size_bytes {
        return Err(PackageError::Invalid(format!(
            "Database size mismatch: expected {}, got {}",
            manifest.database.size_bytes, size
        )));
    }
    let checksum = sha256_file(&database_path)?;
    if checksum != manifest.database.sha256.to_ascii_lowercase() {
        return Err(PackageError::Invalid("Database SHA-256 mismatch".into()));
    }
    validate_sqlite(&database_path, &manifest.pack_type)?;
    Ok(ValidatedPackage {
        manifest,
        database_path,
    })
}

fn validate_manifest(manifest: &DataPackageManifest) -> PackageResult<()> {
    if manifest.manifest_version != PACKAGE_MANIFEST_VERSION {
        return Err(PackageError::Invalid(format!(
            "Unsupported package manifest version: {}",
            manifest.manifest_version
        )));
    }
    validate_token(&manifest.package_id, "package_id")?;
    validate_token(&manifest.version, "version")?;
    if !matches!(
        manifest.pack_type.as_str(),
        "core" | "attribution" | "sharealike"
    ) {
        return Err(PackageError::Invalid("Invalid package pack_type".into()));
    }
    let minimum = manifest
        .min_runtime_api
        .parse::<u64>()
        .map_err(|_| PackageError::Invalid("min_runtime_api must be numeric".into()))?;
    let current = RUNTIME_API_VERSION
        .parse::<u64>()
        .map_err(|_| PackageError::Invalid("Runtime API version is not numeric".into()))?;
    if minimum > current {
        return Err(PackageError::Invalid(format!(
            "Package requires Runtime API {}, current runtime is {}",
            minimum, current
        )));
    }
    validate_single_filename(&manifest.database.file)?;
    let checksum = manifest.database.sha256.as_bytes();
    if checksum.len() != 64 || !checksum.iter().all(u8::is_ascii_hexdigit) {
        return Err(PackageError::Invalid(
            "database.sha256 must be 64 hex characters".into(),
        ));
    }
    if manifest.sources.is_empty() {
        return Err(PackageError::Invalid(
            "Package sources must not be empty".into(),
        ));
    }
    Ok(())
}

fn validate_token(value: &str, field: &str) -> PackageResult<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(PackageError::Invalid(format!("Unsafe {field}")));
    }
    Ok(())
}

fn validate_single_filename(value: &str) -> PackageResult<()> {
    let path = Path::new(value);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(PackageError::Invalid(
            "database.file must be one safe relative filename".into(),
        )),
    }
}

fn validate_sqlite(path: &Path, expected_pack: &str) -> PackageResult<()> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if !integrity.eq_ignore_ascii_case("ok") {
        return Err(PackageError::Invalid(
            "SQLite integrity_check failed".into(),
        ));
    }
    for table in [
        "concepts",
        "localized_names",
        "term_rules",
        "source_versions",
    ] {
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )?;
        if exists != 1 {
            return Err(PackageError::Invalid(format!(
                "Shared database is missing required table {table}"
            )));
        }
    }
    let pack: String = conn
        .query_row(
            "SELECT value FROM build_metadata WHERE key='pack_type'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| PackageError::Invalid("Shared database has no pack_type metadata".into()))?;
    if pack != expected_pack {
        return Err(PackageError::Invalid(format!(
            "Package/database pack mismatch: manifest={expected_pack}, database={pack}"
        )));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> PackageResult<String> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
