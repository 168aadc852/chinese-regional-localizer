# Database / Schema Tasks

Start with:
- the schema or migration being changed;
- code/tests that directly use the affected tables.

Load on demand:
- `docs/DATABASE_SCHEMA.md` for model intent;
- importer/engine guides only if the schema change affects those layers.

Rules:
- preserve provenance and licence-pack separability;
- prefer migrations over silently rewriting historical assumptions;
- keep foreign keys and integrity constraints enabled in tests;
- update schema documentation only for material model changes;
- add/adjust focused tests for constraints and migration behavior.

Do not preload source reviews, project history or unrelated architecture docs.
