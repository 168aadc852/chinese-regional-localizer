use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    package_is_runtime_compatible, validate_package_dir, verify_catalog_package_binding,
    verify_package_manifest_signature, verify_release_catalog, InstalledPackageRef, PackageError,
    PackageStore, ReleaseAuthError, ReleaseCatalog, ReleaseCatalogPackage, SignatureEnvelope,
    TrustedKeySet,
};

pub const DEFAULT_MAX_CATALOG_BYTES: usize = 512 * 1024;
pub const DEFAULT_MAX_SIGNATURE_BYTES: usize = 16 * 1024;
pub const DEFAULT_MAX_MANIFEST_BYTES: usize = 512 * 1024;
pub const DEFAULT_MAX_DATABASE_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug)]
pub enum UpdateError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Http(reqwest::Error),
    Auth(ReleaseAuthError),
    Package(PackageError),
    Invalid(String),
}

impl From<std::io::Error> for UpdateError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for UpdateError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<reqwest::Error> for UpdateError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}
impl From<ReleaseAuthError> for UpdateError {
    fn from(value: ReleaseAuthError) -> Self {
        Self::Auth(value)
    }
}
impl From<PackageError> for UpdateError {
    fn from(value: PackageError) -> Self {
        Self::Package(value)
    }
}

pub type UpdateResult<T> = std::result::Result<T, UpdateError>;

#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub base_url: String,
    pub max_catalog_bytes: usize,
    pub max_signature_bytes: usize,
    pub max_manifest_bytes: usize,
    pub max_database_bytes: usize,
}

impl UpdateConfig {
    pub fn production(base_url: impl Into<String>) -> UpdateResult<Self> {
        let config = Self {
            base_url: base_url.into(),
            max_catalog_bytes: DEFAULT_MAX_CATALOG_BYTES,
            max_signature_bytes: DEFAULT_MAX_SIGNATURE_BYTES,
            max_manifest_bytes: DEFAULT_MAX_MANIFEST_BYTES,
            max_database_bytes: DEFAULT_MAX_DATABASE_BYTES,
        };
        config.base()?;
        Ok(config)
    }

    fn base(&self) -> UpdateResult<Url> {
        let mut url = Url::parse(&self.base_url)
            .map_err(|_| UpdateError::Invalid("Invalid update base URL".into()))?;
        if url.scheme() != "https" {
            return Err(UpdateError::Invalid(
                "Production update base URL must use HTTPS".into(),
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(UpdateError::Invalid(
                "Update base URL must not contain credentials".into(),
            ));
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(UpdateError::Invalid(
                "Update base URL must not contain query or fragment components".into(),
            ));
        }
        if !url.path().ends_with('/') {
            let path = format!("{}/", url.path());
            url.set_path(&path);
        }
        Ok(url)
    }

    fn endpoint(&self, relative: &str) -> UpdateResult<Url> {
        let base = self.base()?;
        let joined = base
            .join(relative)
            .map_err(|_| UpdateError::Invalid("Invalid update endpoint path".into()))?;
        if joined.scheme() != "https"
            || joined.host_str() != base.host_str()
            || joined.port_or_known_default() != base.port_or_known_default()
        {
            return Err(UpdateError::Invalid(
                "Update endpoint escaped the configured HTTPS origin".into(),
            ));
        }
        Ok(joined)
    }
}

pub trait UpdateTransport {
    fn get(&self, url: &Url, max_bytes: usize) -> UpdateResult<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: Client,
}

impl ReqwestTransport {
    pub fn new() -> UpdateResult<Self> {
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(30))
            .user_agent("chinese-regional-localizer/0.1")
            .build()?;
        Ok(Self { client })
    }
}

impl UpdateTransport for ReqwestTransport {
    fn get(&self, url: &Url, max_bytes: usize) -> UpdateResult<Vec<u8>> {
        if url.scheme() != "https" {
            return Err(UpdateError::Invalid(
                "Only HTTPS downloads are allowed".into(),
            ));
        }
        let mut response = self.client.get(url.clone()).send()?;
        if !response.status().is_success() {
            return Err(UpdateError::Invalid(format!(
                "Update server returned HTTP {}",
                response.status()
            )));
        }
        if response
            .content_length()
            .is_some_and(|length| length > max_bytes as u64)
        {
            return Err(UpdateError::Invalid(
                "Update response exceeds size limit".into(),
            ));
        }
        let mut bytes = Vec::new();
        response
            .by_ref()
            .take(max_bytes as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > max_bytes {
            return Err(UpdateError::Invalid(
                "Update response exceeds size limit".into(),
            ));
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateState {
    pub highest_trusted_catalog_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedCatalog {
    catalog: ReleaseCatalog,
    signer_key_id: String,
}

impl AuthenticatedCatalog {
    pub fn catalog(&self) -> &ReleaseCatalog {
        &self.catalog
    }

    pub fn signer_key_id(&self) -> &str {
        &self.signer_key_id
    }

    pub fn package(&self, package_id: &str, version: &str) -> Option<&ReleaseCatalogPackage> {
        self.catalog
            .packages
            .iter()
            .find(|item| item.package_id == package_id && item.version == version)
    }

    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn from_test_parts(catalog: ReleaseCatalog, signer_key_id: String) -> Self {
        Self {
            catalog,
            signer_key_id,
        }
    }
}

pub struct UpdateClient<T: UpdateTransport> {
    transport: T,
    config: UpdateConfig,
    trusted_keys: TrustedKeySet,
    store: PackageStore,
}

impl<T: UpdateTransport> UpdateClient<T> {
    pub fn new(
        transport: T,
        config: UpdateConfig,
        trusted_keys: TrustedKeySet,
        store: PackageStore,
    ) -> UpdateResult<Self> {
        config.base()?;
        Ok(Self {
            transport,
            config,
            trusted_keys,
            store,
        })
    }

    pub fn state(&self) -> UpdateResult<UpdateState> {
        let path = self.update_state_path();
        if !path.exists() {
            return Ok(UpdateState::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn discover(&self, now_unix: i64) -> UpdateResult<AuthenticatedCatalog> {
        let catalog_bytes = self.transport.get(
            &self.config.endpoint("release-catalog.json")?,
            self.config.max_catalog_bytes,
        )?;
        let signature_bytes = self.transport.get(
            &self.config.endpoint("release-catalog.json.sig")?,
            self.config.max_signature_bytes,
        )?;
        let envelope: SignatureEnvelope = serde_json::from_slice(&signature_bytes)?;
        let current_state = self.state()?;
        let catalog = verify_release_catalog(
            &catalog_bytes,
            &envelope,
            &self.trusted_keys,
            now_unix,
            current_state.highest_trusted_catalog_sequence,
        )?;
        let signer_key_id = envelope.key_id.clone();
        if catalog.sequence > current_state.highest_trusted_catalog_sequence {
            self.write_update_state(&UpdateState {
                highest_trusted_catalog_sequence: catalog.sequence,
            })?;
        }
        Ok(AuthenticatedCatalog {
            catalog,
            signer_key_id,
        })
    }

    pub fn install_from_catalog(
        &self,
        catalog: &AuthenticatedCatalog,
        package_id: &str,
        version: &str,
    ) -> UpdateResult<InstalledPackageRef> {
        let entry = catalog.package(package_id, version).ok_or_else(|| {
            UpdateError::Invalid("Requested package/version is not in authenticated catalog".into())
        })?;
        if !package_is_runtime_compatible(entry) {
            return Err(UpdateError::Invalid(
                "Requested package is not compatible with this runtime".into(),
            ));
        }

        let prefix = format!("packages/{package_id}/{version}/");
        let manifest_bytes = self.transport.get(
            &self.config.endpoint(&(prefix.clone() + "package.json"))?,
            self.config.max_manifest_bytes,
        )?;
        let signature_bytes = self.transport.get(
            &self
                .config
                .endpoint(&(prefix.clone() + "package.json.sig"))?,
            self.config.max_signature_bytes,
        )?;
        let envelope: SignatureEnvelope = serde_json::from_slice(&signature_bytes)?;
        verify_package_manifest_signature(&manifest_bytes, &envelope, &self.trusted_keys)?;
        let manifest = verify_catalog_package_binding(entry, &manifest_bytes)?;

        let database_bytes = self.transport.get(
            &self.config.endpoint(&(prefix + &manifest.database.file))?,
            self.config.max_database_bytes,
        )?;
        if database_bytes.len() as u64 != manifest.database.size_bytes {
            return Err(UpdateError::Invalid(
                "Downloaded database size does not match signed manifest".into(),
            ));
        }

        let staging = self.staging_path(package_id, version);
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir_all(&staging)?;
        let result = (|| -> UpdateResult<InstalledPackageRef> {
            fs::write(staging.join("package.json"), &manifest_bytes)?;
            fs::write(staging.join(&manifest.database.file), &database_bytes)?;
            validate_package_dir(&staging)?;
            Ok(self.store.install(&staging)?)
        })();
        let _ = fs::remove_dir_all(&staging);
        result
    }

    fn update_state_path(&self) -> PathBuf {
        self.store.root().join("update-state.json")
    }

    fn staging_path(&self, package_id: &str, version: &str) -> PathBuf {
        self.store.root().join(format!(
            ".network-stage-{package_id}-{version}-{}",
            std::process::id()
        ))
    }

    fn write_update_state(&self, state: &UpdateState) -> UpdateResult<()> {
        fs::create_dir_all(self.store.root())?;
        let target = self.update_state_path();
        let temp = self.store.root().join("update-state.json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(state)?)?;
        fs::rename(temp, target)?;
        Ok(())
    }
}

pub fn validate_update_base_url(base_url: &str) -> UpdateResult<()> {
    UpdateConfig::production(base_url).map(|_| ())
}

pub fn update_state_path(store_root: &Path) -> PathBuf {
    store_root.join("update-state.json")
}
