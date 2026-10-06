# Source Manifest Schema

`data-registry/sources.yaml` is the machine-readable policy manifest for external data ingestion.

Human-readable legal/research notes remain in the corresponding `data-registry/*.md` files. The YAML manifest does not replace those records; it converts the approved conclusions into fields that AI tools and importers can enforce.

## Canonical fields

Each `sources[]` entry must contain:

- `id` — stable machine identifier; do not reuse for a different source/scope.
- `name` — human-readable source or source subset name.
- `status` — one of `approved`, `approved_with_conditions`, `pending_review`, `reference_only`, `rejected`.
- `pack` — one of `core`, `attribution`, `sharealike`, `reference_only`, `pending`.
- `licence` — SPDX-like identifier or clear licence name when verified; `null` when unresolved.
- `commercial_use` — `true`, `false`, or `null` when unresolved.
- `modification_allowed` — `true`, `false`, or `null` when unresolved.
- `redistribution_allowed` — `true`, `false`, or `null` when unresolved.
- `attribution_required` — `true`, `false`, or `null` when unresolved.
- `share_alike` — `true`, `false`, or `null` when unresolved.
- `ingest_allowed` — hard allow/deny flag for automated import.
- `ingest_scope` — exact files/namespaces/subsets allowed for ingestion.
- `excluded_scope` — files/content explicitly excluded or not yet proven safe.
- `review_record` — path to the detailed human-readable review file.

## Hard safety rules

1. `pending_review`, `reference_only`, and `rejected` entries must have `ingest_allowed: false`.
2. An importer must not infer permission from `status`, source name, public accessibility, or repository visibility alone; it must enforce `ingest_allowed` and `ingest_scope`.
3. If a source contains multiple licences, split it into multiple manifest entries when practical. Example: Rime Cantonese main content and `jyut6ping3.maps` are separate entries.
4. A source marked `core` must not silently contain ShareAlike or non-commercial data.
5. A `sharealike` source must remain in a separately releasable data pack unless a later legal/architectural decision explicitly changes this.
6. `null` means unresolved; it must never be interpreted as permission.
7. Every ingested record must retain enough provenance to identify its manifest `id`, upstream source/revision/version and transformation history.

## Pack meanings

### `core`
Permissive/public-domain style sources that can participate in the standard core database, subject to notices recorded in their source reviews.

### `attribution`
Commercial-capable sources that require attribution but do not impose a general ShareAlike requirement on the data pack.

### `sharealike`
Sources with CC BY-SA, ODbL, or comparable reciprocal obligations. Keep isolated from permissive-only releases.

### `reference_only`
Useful for human checking but excluded from redistributable standard databases.

### `pending`
Rights, licence version, or file scope are unresolved. Automated ingestion is prohibited.

## Change control

When a source review changes:

1. update the relevant `data-registry/<source>.md` evidence first;
2. update `data-registry/sources.yaml` to match the evidence;
3. update `docs/DATA_SOURCES.md` / `docs/REVIEW_PROGRESS.md` if status or packaging changed;
4. record material architecture/licensing decisions in `docs/DECISIONS.md`;
5. update `PROJECT_STATE.md` when project progress changes materially.

Do not edit the YAML status to make development easier without updating the evidence record.