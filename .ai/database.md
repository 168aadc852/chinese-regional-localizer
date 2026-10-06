# Database / Schema Tasks

Start with:
- the schema or migration being changed;
- code/tests that directly use the affected tables.

Load on demand:
- `docs/DATABASE_SCHEMA.md` for model intent;
- importer/engine guides only if the schema change affects those layers.

Rules:
- preserve provenance and licence-pack separability;
- keep historical schema files stable; add a new base schema/migration rather than silently rewriting old assumptions;
- source updates must preserve history and identify one current snapshot per resource;
- keep foreign keys and integrity constraints enabled in tests;
- indexes should match real runtime/importer query shapes;
- update schema documentation only for material model changes;
- add focused migration/update/idempotency tests.

Current base schema: `schema/sqlite-v0.2.sql`. Historical v0.1 databases upgrade through migration 0003.
