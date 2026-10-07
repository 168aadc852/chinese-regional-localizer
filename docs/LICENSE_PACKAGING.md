# Licence and Packaging Model

Project-authored software is licensed under Apache-2.0 (`LICENSE`). Third-party data does not inherit that licence; see `LICENSE_SCOPE.md`, source reviews and manifest metadata.

## Data packs

Canonical SQLite releases remain separated by licence obligation:

- `core` — permissive/public-domain style data;
- `attribution` — commercial-capable data requiring attribution;
- `sharealike` — reciprocal/copyleft-style data;
- user-local data — never merged into canonical downloaded packs.

The importer layer now enforces `build_metadata.pack_type`: importing an attribution/share-alike source into an existing core database is rejected rather than silently creating a mixed-licence artifact.

## Release rules

A redistributable build must:

- contain only approved, exact machine-allowed resources;
- preserve source/version/checksum provenance;
- include required third-party licence/attribution notices;
- keep reciprocal data in its designated pack;
- exclude pending/reference-only/rejected sources;
- pass tests and SQLite integrity checks.

A repository-wide software licence never overrides third-party data terms.
