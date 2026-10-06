# Coding Tasks

Read only what the change touches.

Start with:
- the target source file(s);
- the nearest relevant test file(s);
- the GitHub Issue/task description, if one exists.

Load on demand:
- `PROJECT_STATE.md` only if current phase/status matters;
- `.ai/database.md` if schema/query behavior changes;
- `.ai/engine.md` if localization semantics change;
- `.ai/importer.md` if external-data ingestion changes.

Do not preload README, product docs, data-source reviews or project history for ordinary code changes.

Keep behavior changes covered by tests. Do not silently alter established architecture or licensing boundaries.
