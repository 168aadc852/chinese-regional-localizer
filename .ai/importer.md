# Importer Tasks

Start with:
- the importer being changed;
- its tests/fixtures;
- the specific source record in `data-registry/`;
- the matching `data-registry/sources.yaml` entry.

Load on demand:
- `docs/DATABASE_SCHEMA.md` only if schema usage changes;
- source-specific importer docs only for that importer;
- `.ai/database.md` if a schema migration is required.

Requirements:
- `ingest_allowed` must be true;
- the exact resource must be listed in `ingest_resources`;
- obey reviewed scope/exclusions and expected licence pack;
- preserve resource key, revision/version, source URL, checksum and retrieval provenance;
- repeat imports must be idempotent;
- changed snapshots must supersede old runtime data without deleting history;
- keep network access explicit; tests use local fixtures;
- reject malformed, out-of-scope or falsely identified input rather than guessing.

Do not preload unrelated source reviews or importer docs.
