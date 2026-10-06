# Wikidata Entity Importer — Phase 1C

This importer is the project's first real **entity/localized-name** ingestion path.

## Source scope

Only Wikidata structured entity JSON covered by the reviewed CC0 scope is used.

The targeted interface is Wikidata's stable Linked Data endpoint:

```text
https://www.wikidata.org/wiki/Special:EntityData/<QID>.json
```

The importer can explicitly download selected QIDs, or read an already downloaded local EntityData JSON document.

## Why this layer is separate from OpenCC

OpenCC contributes deterministic script/regional term mappings.

Wikidata contributes stable real-world entity identity:

- QID
- localized labels
- localized aliases
- revision/provenance

This means a future localization engine can distinguish an entity such as a person or film from an ordinary word before deciding which regional name to use.

## Locale policy

Initial accepted Wikidata language codes:

- `en`
- `mul`
- `zh`
- `zh-hans`
- `zh-hant`
- `zh-cn`
- `zh-hk`
- `zh-tw`

Stored locale normalization:

- `zh-cn` → `zh-CN`
- `zh-hk` → `zh-HK`
- `zh-tw` → `zh-TW`
- `zh-hans` → `zh-Hans`
- `zh-hant` → `zh-Hant`

Hard rule: the importer **does not synthesize a missing regional label** from generic `zh`, `zh-Hans` or `zh-Hant`. If Wikidata does not supply `zh-HK`, for example, no `zh-HK` row is invented by this importer.

Other sources/resolution logic may provide additional evidence later.

## Entity type policy

Phase 1C requires an explicit QID-to-concept-type mapping rather than inferring type from labels.

Example:

```json
{
  "Q35332": "person",
  "Q108839994": "film"
}
```

A later importer version may derive or validate types from structured statements such as `instance of`, but that is outside this first step.

## Database mapping

For each QID:

- `concepts` stores the project concept and explicit concept type;
- `external_ids` stores namespace `wikidata` + the QID;
- `localized_names` stores labels as `preferred` and aliases as `alias`;
- `name_evidence` stores Wikidata evidence and revision provenance;
- `source_versions` stores one exact source version per imported entity.

## Provenance

The importer records:

- QID;
- `lastrevid`;
- entity `modified` timestamp when present;
- retrieval timestamp;
- revision-pinned upstream URL for non-fixture data;
- SHA-256 of a canonical JSON serialization of the imported entity object.

The evidence transformation note explicitly states that only locale-code normalization was performed; no name translation/fallback is fabricated.

## Validation

The importer rejects:

- missing top-level `entities` object;
- malformed QIDs;
- entity key / entity `id` mismatch;
- non-item entities;
- missing or invalid `lastrevid`;
- malformed labels/aliases;
- missing explicit entity type mapping.

Unknown language codes are ignored in Phase 1C rather than guessed.

## Test fixtures

`data/fixtures/wikidata/entities.json` is a **small hand-authored format fixture**, not a claim that its revision numbers are current Wikidata revisions.

Fixture provenance therefore uses `fixture://` URLs in tests.

The fixture demonstrates a person and film with distinct CN/HK/TW localized names so schema behavior can be tested offline.

## Local fixture run

```bash
python scripts/import_wikidata_entities.py \
  --input data/fixtures/wikidata/entities.json \
  --type-map data/fixtures/wikidata/entity_types.json \
  --fixture-mode \
  --db build/wikidata.sqlite \
  --reset
```

## Explicit selected-entity download

```bash
python scripts/import_wikidata_entities.py \
  --download-qid Q35332 \
  --download-qid Q108839994 \
  --type-map data/fixtures/wikidata/entity_types.json \
  --db build/wikidata.sqlite \
  --reset
```

This targeted download mode is for database building/updating. Normal application localization remains offline-first.

## Future work

- type validation from structured claims;
- batch extraction from official Wikidata dumps;
- alias ranking/deduplication across multiple sources;
- entity conflict/resolution policy;
- incremental entity refresh using recorded revisions.
