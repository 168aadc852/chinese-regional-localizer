# D-008 — Machine-readable source manifest controls ingestion

Date: 2026-10-06

`data-registry/sources.yaml` is the ingestion policy for external data.

Rules:
- automated import requires `ingest_allowed: true`;
- `null` permission fields mean unresolved, never permitted;
- obey `ingest_scope` and `excluded_scope`;
- keep multi-licence sources split when practical;
- keep `core`, `attribution`, `sharealike`, `reference_only` and `pending` packs separable.

Detailed evidence stays in the specific `data-registry/*.md` record.
