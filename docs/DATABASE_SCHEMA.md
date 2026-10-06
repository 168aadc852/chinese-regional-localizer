# Database Schema v0.2

Schema v0.2 is the update-safe reference SQLite model. `schema/sqlite-v0.1.sql` remains the historical Phase 0.5 baseline; new databases use `schema/sqlite-v0.2.sql`. Existing v0.1 databases are upgraded by `schema/migrations/0003_core_hardening.sql`.

## Design goals

- keep source/version provenance for every imported claim;
- preserve history without letting old snapshots remain active;
- keep core, attribution and share-alike packs separately releasable;
- support deterministic offline entity/term lookup;
- allow conflicting claims to coexist while the resolver applies no-guess behavior;
- keep user-local overrides outside canonical downloaded databases.

## `sources`

One row per manifest source. The database copy is refreshed from `data-registry/sources.yaml` on every production import, rather than only when a database is first created.

Important policy fields include status, pack, licence, rights flags, `ingest_allowed`, human-readable scope and the review-record path.

## `source_versions`

Each exact upstream resource snapshot is stored with:

- `source_id`;
- `resource_key` — stable project key such as `opencc:TWPhrases.txt` or `wikidata:Q35332`;
- `is_current` — exactly which snapshot the runtime should treat as current for that resource;
- version/revision identifiers;
- publication/retrieval time;
- upstream URL;
- SHA-256;
- notes.

Importers reuse an identical current revision/checksum instead of inserting duplicate snapshots. A changed snapshot marks the previous resource version non-current while preserving history.

## `concepts` and `external_ids`

`concepts` represents lexical concepts and real-world entities. `external_ids` holds stable identifiers such as Wikidata QIDs or Unicode code points. External identifiers remain unique by `(namespace, external_value)`.

## `localized_names`

Stores normalized display names independently from evidence. A row may remain historically present after an upstream rename; runtime resolution considers whether it still has current evidence.

The `is_preferred` flag describes the claim type, not snapshot freshness. Freshness comes from evidence joined to current source versions.

## `name_evidence`

Connects localized names to source/version provenance. Runtime entity resolution ignores evidence whose `source_version_id` has been superseded. `current_name_evidence` exposes the current subset.

## `term_rules`

Stores deterministic conversion rules with source/target locale, text, priority, provenance and `active` state.

OpenCC refreshes deactivate rules belonging to superseded resource versions. The primary lookup index is ordered around the actual runtime query:

`(source_locale, target_locale, active, source_text, priority)`.

`context_constraint` stores JSON. Metadata-only objects are allowed; executable restrictions live under a `constraints` object. The reference engine currently enforces domain, preceding/following text and optional ASCII word-boundary constraints.

## `pronunciations`

Added by migration `0002_pronunciations.sql`. Pronunciation rows preserve source-version provenance. `current_pronunciations` filters out superseded source versions after migration 0003.

## `build_metadata`

Important keys now include:

- `schema_version`;
- `pack_type`;
- source-specific last revision/checksum keys;
- fixture/build markers where applicable.

`pack_type` is an enforcement boundary: a core database cannot silently accept attribution/share-alike data, and vice versa.

## User-local data

Personal protected terms and user overrides must remain in a separate local database such as `user_dictionary.sqlite`. Canonical packs must be replaceable without deleting user preferences.

## Importer safety contract

A production importer must:

1. validate the source manifest;
2. require `ingest_allowed: true`;
3. require an exact `ingest_resources` identifier;
4. enforce the expected licence pack;
5. sync manifest metadata into SQLite;
6. validate upstream format/identity;
7. record exact resource/revision/URL/checksum/retrieval provenance;
8. make repeat imports idempotent;
9. supersede, rather than silently coexist with, an older current snapshot;
10. run `PRAGMA integrity_check` before success.
