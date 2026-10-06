# Phase 0.5 SQLite Proof of Concept

Status: **completed and CI-validated on 2026-10-06**.

This proof of concept validates the project architecture before production importers or application UI are built.

## What it proves

- `data-registry/sources.yaml` can act as a machine-readable ingestion policy.
- Sources marked `pending_review`, `reference_only` or `rejected` are blocked from data ingestion.
- Source metadata can still be recorded in SQLite even when the source is blocked from ingestion.
- CN / HK / TW terminology and entity names can coexist in one schema.
- Every imported name/rule can retain source/version/provenance information.
- The database can keep permissive, attribution and ShareAlike source metadata distinct.
- GitHub Actions successfully runs the unit tests and builds the demo database.

## Important fixture warning

`data/fixtures/poc_records.json` is hand-authored demonstration data. It is not an authoritative extract from OpenCC, Wikidata or Words.hk. Evidence rows are marked `poc_fixture` and use `fixture://` URLs so they cannot be mistaken for real upstream provenance.

## Run locally

Requirements:

- Python 3.11+
- `pip install -r requirements-dev.txt`

Then run:

```bash
python scripts/poc_builder.py --demo
```

The generated SQLite file is written to `build/localizer-poc.sqlite` and is ignored by Git.

## Run tests

```bash
python -m unittest discover -s tests -v
```

GitHub Actions runs the same tests on pushes and pull requests to `main`.

## What comes next

Production importers should now be implemented one source at a time. Each importer must:

1. read `data-registry/sources.yaml`;
2. refuse blocked sources;
3. enforce `ingest_scope` / `excluded_scope`;
4. pin a source version/snapshot;
5. record upstream IDs/URLs/checksums/retrieval time;
6. preserve pack/licence separation;
7. add source-specific tests.
