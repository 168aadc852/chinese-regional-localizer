# D-008 — Machine-readable source manifest controls ingestion

Date: 2026-10-06  
Hardened: 2026-10-07

`data-registry/sources.yaml` is the ingestion policy for external data.

Rules:
- automated import requires `ingest_allowed: true`;
- implemented production importers additionally require an exact resource identifier in `ingest_resources`;
- human-readable `ingest_scope` is audit context, not a substitute for the exact resource gate;
- `null` permission fields mean unresolved, never permitted;
- sources with unresolved adaptation rights are not enabled for derived-database ingestion;
- keep multi-licence sources split when practical;
- keep `core`, `attribution`, `sharealike`, `reference_only` and `pending` packs separable;
- a SQLite build records/enforces one `pack_type` unless an explicit later architecture decision defines a lawful composite release.

Detailed evidence stays in the specific `data-registry/*.md` record.
