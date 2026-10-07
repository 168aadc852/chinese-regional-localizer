# Wikidata Entity Importer — Phase 2C

This importer ingests selected CC0 Wikidata item EntityData for stable identity and locale-specific names.

## Machine scope

The production importer requires exact resource policy `wikidata:entity-json:item`. Arbitrary prose/non-item namespace content is not accepted through this path.

Accepted language codes remain `en`, `mul`, `zh`, `zh-hans`, `zh-hant`, `zh-cn`, `zh-hk`, `zh-tw`, normalized to project locale casing. Missing regional labels are never synthesized from generic Chinese labels.

## Entity types

Each imported QID still requires an explicit project concept-type mapping. Type inference from Wikidata statements remains future work.

## Revision-aware refresh

Each QID has its own `resource_key` (`wikidata:<QID>`). An identical revision/checksum is idempotent. A changed revision marks the prior QID snapshot non-current while preserving its evidence.

The engine resolves only names backed by current evidence. Therefore a changed preferred label or removed alias no longer competes with the current snapshot merely because its historical `localized_names` row remains stored.

## Provenance

Per entity the importer records QID, `lastrevid`, modified timestamp when present, retrieval timestamp, revision-specific/fixture URL, canonical JSON SHA-256 and one source-version row. Name evidence records the exact normalized locale/name claim and revision.

## Conservative runtime matching

Importing a Wikidata name does not automatically guarantee that every occurrence of that surface in prose is an entity. The localization engine applies additional conservative surface rules to reduce common-word false positives.

## Future work

- structured type validation;
- dump-scale extraction/incremental feeds;
- richer cross-source ranking;
- contextual entity disambiguation;
- measured large-corpus performance work before production Rust parity.
