# Licence and Packaging Model

Project-authored software is licensed under Apache-2.0 (`LICENSE`). Third-party data does not inherit that licence; see `LICENSE_SCOPE.md`, source reviews and manifest metadata.

## Data packs

Canonical SQLite releases remain separated by licence obligation:

- `core` — permissive/public-domain style data;
- `attribution` — commercial-capable data requiring attribution;
- `sharealike` — reciprocal/copyleft-style data;
- user-local data — never merged into canonical downloaded packs.

The importer layer enforces `build_metadata.pack_type`: importing an attribution/share-alike source into an existing core database is rejected rather than silently creating a mixed-licence artifact.

## Release rules

A redistributable build must:

- contain only approved, exact machine-allowed resources;
- preserve source/version/checksum provenance;
- include required third-party licence/attribution notices;
- keep reciprocal data in its designated pack;
- exclude pending/reference-only/rejected sources;
- pass tests and SQLite integrity checks.

A repository-wide software licence never overrides third-party data terms.

## Phase 3F package gate

`scripts/build_data_package.py` adds a second governance check immediately before packaging. It re-reads `data-registry/sources.yaml` and refuses a package when a current database source is blocked, unknown, outside its exact `ingest_resources` allow-list, or assigned to a different licence pack.

Each release directory includes `package.json` with the pack type, runtime compatibility, database size/SHA-256 and current source-version summary. The Rust installer verifies these values again before staging or activating a package.

SHA-256 is used for integrity/corruption detection only. It does not replace publisher authentication or future release signing.
