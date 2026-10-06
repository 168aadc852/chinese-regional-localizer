# Importer Tasks

Start with:
- the importer being changed;
- its tests/fixtures;
- the specific source record in `data-registry/`;
- the matching `data-registry/sources.yaml` entry.

Load on demand:
- `docs/DATABASE_SCHEMA.md` only if schema usage changes;
- source-specific docs such as `docs/OPENCC_IMPORTER.md` only for that importer;
- `.ai/database.md` if a schema migration is required.

Requirements:
- `ingest_allowed` must be true;
- obey `ingest_scope` and `excluded_scope`;
- preserve revision/version, source URL and checksum provenance;
- keep network access explicit; tests should use local fixtures;
- reject malformed or out-of-scope input rather than guessing.

Do not preload unrelated source reviews or importer docs.
