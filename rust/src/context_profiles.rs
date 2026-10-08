//! Usage-context structure only: no terminology, runtime or desktop-settings integration.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub const CONTEXT_PROFILE_SCHEMA_VERSION: u64 = 1;
pub const CONTEXT_PROFILE_FILE: &str = "context-profiles.json";
pub const MAX_CONTEXT_PROFILE_BYTES: usize = 1024 * 1024;
pub const MAX_CONTEXT_PROFILES: usize = 1024;

const BUILT_INS: [(&str, &str); 7] = [
    ("general", "General"),
    ("technology-software", "Technology / Software"),
    ("banking-finance", "Banking / Finance"),
    ("business-marketing", "Business / Marketing"),
    ("legal", "Legal"),
    ("education", "Education"),
    (
        "government-public-administration",
        "Government / Public Administration",
    ),
];

#[derive(Debug)]
pub enum ContextProfileError {
    Io(std::io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u64),
    Invalid(String),
    UnknownContext(String),
    DisabledContext(String),
    TooLarge,
}

impl std::fmt::Display for ContextProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Context profile storage error: {error}"),
            Self::Json(error) => write!(f, "Invalid context profile document: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "Unsupported context profile schema version: {version}")
            }
            Self::Invalid(message) => write!(f, "Invalid context profile: {message}"),
            Self::UnknownContext(id) => write!(f, "Context does not exist: {id}"),
            Self::DisabledContext(id) => write!(f, "Context is disabled: {id}"),
            Self::TooLarge => write!(f, "Context profile document exceeds the size limit"),
        }
    }
}

impl std::error::Error for ContextProfileError {}

impl From<std::io::Error> for ContextProfileError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ContextProfileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub type ContextProfileResult<T> = Result<T, ContextProfileError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContextProfile {
    pub context_id: String,
    pub display_name: String,
    #[serde(default = "default_parent_context_id")]
    pub parent_context_id: Option<String>,
    pub built_in: bool,
    pub enabled: bool,
}

fn default_parent_context_id() -> Option<String> {
    Some("general".into())
}

impl ContextProfile {
    /// New custom profiles inherit General unless the caller explicitly changes it.
    pub fn custom(context_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            context_id: context_id.into(),
            display_name: display_name.into(),
            parent_context_id: default_parent_context_id(),
            built_in: false,
            enabled: true,
        }
    }
}

fn built_in_profiles() -> Vec<ContextProfile> {
    BUILT_INS
        .iter()
        .map(|(id, name)| ContextProfile {
            context_id: (*id).into(),
            display_name: (*name).into(),
            parent_context_id: (*id != "general").then(|| "general".into()),
            built_in: true,
            enabled: true,
        })
        .collect()
}

// One version for the whole document avoids contradictory per-profile versions.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfilesV1 {
    schema_version: u64,
    profiles: Vec<ContextProfile>,
}

/// A fully validated, immutable snapshot. Rebuild it to apply edits atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextProfiles {
    profiles: BTreeMap<String, ContextProfile>,
}

impl Default for ContextProfiles {
    fn default() -> Self {
        Self::from_profiles(built_in_profiles()).expect("valid product defaults")
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.as_bytes()[0].is_ascii_lowercase()
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

impl ContextProfiles {
    /// Add custom definitions to the seven defaults; reserved IDs cannot be replaced.
    pub fn with_custom_profiles(custom: Vec<ContextProfile>) -> ContextProfileResult<Self> {
        if custom.iter().any(|profile| profile.built_in) {
            return Err(ContextProfileError::Invalid(
                "custom profiles cannot claim built-in identity".into(),
            ));
        }
        let mut profiles = built_in_profiles();
        profiles.extend(custom);
        Self::from_profiles(profiles)
    }

    /// Validate an entire document/edit, including disabled profiles and all ancestors.
    /// Built-in names/enabled flags may change, but their IDs and parent links may not.
    pub fn from_profiles(profiles: Vec<ContextProfile>) -> ContextProfileResult<Self> {
        if profiles.len() > MAX_CONTEXT_PROFILES {
            return Err(ContextProfileError::TooLarge);
        }
        let mut indexed = BTreeMap::new();
        for profile in profiles {
            if !valid_id(&profile.context_id)
                || profile
                    .parent_context_id
                    .as_deref()
                    .is_some_and(|id| !valid_id(id))
                || profile.display_name.trim().is_empty()
                || profile.display_name.trim() != profile.display_name
                || profile.display_name.chars().count() > 128
                || profile.display_name.chars().any(char::is_control)
            {
                return Err(ContextProfileError::Invalid(
                    "invalid ID or display name".into(),
                ));
            }
            let reserved = BUILT_INS.iter().any(|(id, _)| *id == profile.context_id);
            if profile.built_in != reserved {
                return Err(ContextProfileError::Invalid(format!(
                    "reserved or unrecognized built-in ID: {}",
                    profile.context_id
                )));
            }
            if reserved {
                let expected_parent = (profile.context_id != "general").then_some("general");
                if profile.parent_context_id.as_deref() != expected_parent {
                    return Err(ContextProfileError::Invalid(
                        "built-in parent links cannot be changed".into(),
                    ));
                }
            }
            let id = profile.context_id.clone();
            if indexed.insert(id.clone(), profile).is_some() {
                return Err(ContextProfileError::Invalid(format!(
                    "duplicate context ID: {id}"
                )));
            }
        }
        for (id, _) in BUILT_INS {
            if !indexed.contains_key(id) {
                return Err(ContextProfileError::Invalid(format!(
                    "missing built-in context: {id}"
                )));
            }
        }
        let validated = Self { profiles: indexed };
        for id in validated.profiles.keys() {
            validated.chain(id, false)?;
        }
        Ok(validated)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &ContextProfile> {
        self.profiles.values()
    }

    pub fn get(&self, context_id: &str) -> Option<&ContextProfile> {
        self.profiles.get(context_id)
    }

    /// Specific context first, then each ancestor. A disabled ancestor makes the
    /// whole chain unavailable; it is never silently skipped or replaced.
    pub fn resolve_chain(&self, context_id: &str) -> ContextProfileResult<Vec<&ContextProfile>> {
        self.chain(context_id, true)
    }

    fn chain(
        &self,
        context_id: &str,
        require_enabled: bool,
    ) -> ContextProfileResult<Vec<&ContextProfile>> {
        let mut chain = Vec::new();
        let mut visited = BTreeSet::new();
        let mut next = Some(context_id);
        while let Some(id) = next {
            if !visited.insert(id) {
                return Err(ContextProfileError::Invalid(format!(
                    "inheritance cycle at: {id}"
                )));
            }
            let profile = self
                .get(id)
                .ok_or_else(|| ContextProfileError::UnknownContext(id.into()))?;
            if require_enabled && !profile.enabled {
                return Err(ContextProfileError::DisabledContext(id.into()));
            }
            chain.push(profile);
            next = profile.parent_context_id.as_deref();
        }
        Ok(chain)
    }

    /// Version dispatch/migration seam: v1 is the first format; no fake v0 upgrade.
    /// Parse the original bytes after dispatch to reject duplicate JSON fields.
    pub fn from_json(bytes: &[u8]) -> ContextProfileResult<Self> {
        if bytes.len() > MAX_CONTEXT_PROFILE_BYTES {
            return Err(ContextProfileError::TooLarge);
        }
        let value: serde_json::Value = serde_json::from_slice(bytes)?;
        match value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
        {
            Some(CONTEXT_PROFILE_SCHEMA_VERSION) => {
                let document: ProfilesV1 = serde_json::from_slice(bytes)?;
                Self::from_profiles(document.profiles)
            }
            Some(version) => Err(ContextProfileError::UnsupportedVersion(version)),
            None => Err(ContextProfileError::Invalid(
                "schema_version must be an unsigned integer".into(),
            )),
        }
    }

    /// Canonical ID order makes loading/resaving v1 a deterministic identity migration.
    pub fn to_json(&self) -> ContextProfileResult<Vec<u8>> {
        let bytes = serde_json::to_vec_pretty(&ProfilesV1 {
            schema_version: CONTEXT_PROFILE_SCHEMA_VERSION,
            profiles: self.profiles().cloned().collect(),
        })?;
        if bytes.len() > MAX_CONTEXT_PROFILE_BYTES {
            return Err(ContextProfileError::TooLarge);
        }
        Ok(bytes)
    }
}

/// An opt-in local store, not wired to Desktop or localization. The caller owns
/// the directory and serializes edits; there is no multi-process writer protocol.
#[derive(Debug, Clone)]
pub struct ContextProfileStore {
    directory: PathBuf,
}

impl ContextProfileStore {
    pub fn new(directory: impl AsRef<Path>) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(CONTEXT_PROFILE_FILE)
    }

    fn read_existing(&self) -> ContextProfileResult<Option<ContextProfiles>> {
        let file = match File::open(self.path()) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_CONTEXT_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        ContextProfiles::from_json(&bytes).map(Some)
    }

    /// Missing document returns built-ins without creating any files. Bad existing
    /// documents return an error, never a silently repaired/defaulted snapshot.
    pub fn load(&self) -> ContextProfileResult<ContextProfiles> {
        Ok(self.read_existing()?.unwrap_or_default())
    }

    /// Refuse to overwrite invalid/unsupported documents. Save a complete validated
    /// snapshot via synced same-directory atomic replacement, including on Windows.
    pub fn save(&self, profiles: &ContextProfiles) -> ContextProfileResult<()> {
        self.read_existing()?;
        let bytes = profiles.to_json()?;
        fs::create_dir_all(&self.directory)?;
        let mut staged = NamedTempFile::new_in(&self.directory)?;
        staged.write_all(&bytes)?;
        staged.as_file().sync_all()?;
        staged
            .persist(self.path())
            .map_err(|error| ContextProfileError::Io(error.error))?;
        Ok(())
    }
}
