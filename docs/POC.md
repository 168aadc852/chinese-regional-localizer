# Phase 0.5 SQLite Proof of Concept

Status: **historical milestone; completed and CI-validated on 2026-10-06**.

This PoC established the provenance-first data model, manifest gating and offline fixture workflow. It is retained as historical architecture context; current implementation status is Phase 2C and uses schema v0.2/update-safe importers.

The original fixture builder remains available:

```bash
python scripts/poc_builder.py --demo
```

Its fixture records are non-authoritative and use `fixture://` provenance.

Current production-importer requirements are stricter than the original Phase 0.5 contract: exact `ingest_resources`, licence-pack enforcement, update idempotency/current source versions, real timestamps and source-specific validation are now required. See `docs/DATABASE_SCHEMA.md`, `docs/SOURCE_MANIFEST_SCHEMA.md` and the current importer docs.
