# Shared Data Package Format v1

Phase 3F defines an offline-safe package format for redistributable shared SQLite databases.

A package is a directory containing exactly the release manifest plus the referenced database artifact, for example:

```text
package/
  package.json
  regional.sqlite
```

## `package.json`

Manifest version `1` contains:

- `manifest_version` — package manifest schema version;
- `package_id` — stable package family identifier;
- `version` — package release version;
- `pack_type` — one of `core`, `attribution`, `sharealike`;
- `min_runtime_api` — minimum supported Rust Runtime API version;
- `created_at` — package creation timestamp;
- `database.file` — one safe relative filename only;
- `database.size_bytes` — exact byte length;
- `database.sha256` — SHA-256 integrity checksum;
- `sources` — current source/resource/version/checksum provenance summary extracted from SQLite.

## Build-time governance

`scripts/build_data_package.py` refuses to create a package unless:

- SQLite `PRAGMA integrity_check` succeeds;
- the database has a redistributable `build_metadata.pack_type`;
- every current source exists in `data-registry/sources.yaml`;
- every current source is machine-approved for ingestion and is not pending/reference-only/rejected;
- each source's manifest pack equals the database pack;
- current resource keys obey the exact `ingest_resources` allow-list where one exists.

This is an additional release gate. It does not replace importer-time governance.

## Runtime validation

The Rust package validator independently checks:

- manifest version and runtime compatibility;
- safe package/version identifiers;
- safe single-file relative database path;
- expected byte size and SHA-256;
- SQLite integrity;
- required runtime tables;
- package/database `pack_type` agreement.

The checksum detects corruption or accidental modification. It does **not** prove publisher identity. Cryptographic release signing is intentionally future work.

## Versioned local store

`PackageStore` installs packages under:

```text
<store>/packages/<package_id>/<version>/
```

The active pointer is stored separately in `state.json` with `current` and `previous` package references.

Installation follows this order:

1. validate the incoming package;
2. copy it to a staging directory;
3. validate the staged copy again;
4. rename staging into its immutable version directory;
5. only then activate the version.

The currently active database is never overwritten in place. Failed validation therefore leaves the previous working version active.

`rollback()` validates the previous installed package before swapping `current` and `previous`.

## Phase 3F boundary

Phase 3F is offline only. It does not implement:

- HTTP update discovery/download;
- publisher signatures;
- automatic cleanup of old versions;
- installer/signing integration;
- mobile delivery.

Those layers must build on this validated package/store contract rather than bypass it.
