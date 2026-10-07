# Source Manifest Schema

`data-registry/sources.yaml` is the machine-readable policy manifest for external data ingestion. Human-readable evidence stays in the corresponding `data-registry/*.md` file.

## Canonical fields

Every source entry contains identity/status/pack/licence fields, commercial/modification/redistribution rights, attribution/share-alike flags, `ingest_allowed`, human-readable `ingest_scope` / `excluded_scope`, and `review_record`.

### `ingest_resources`

Implemented production importers additionally require an exact machine-readable allow-list:

```yaml
ingest_resources:
  - opencc:STPhrases.txt
  - opencc:STCharacters.txt
```

An importer must call the resource gate with one exact identifier. If a source has no `ingest_resources` list, no new production importer may infer a machine-safe subset from prose alone; the manifest must first be updated from reviewed evidence.

Current examples:

- OpenCC: one identifier per reviewed dictionary file;
- Wikidata: `wikidata:entity-json:item`;
- LSHK: `lshk:list.tsv`.

## Hard safety rules

1. Pending, reference-only and rejected sources must have `ingest_allowed: false`.
2. `ingest_allowed` alone is insufficient for implemented production imports; the exact resource must also be allowed.
3. Mixed-licence source families should be split into separate manifest IDs where practical.
4. Core databases must not silently contain share-alike or attribution-pack data.
5. `null` never means permission.
6. Every imported record must retain source/version/resource provenance.
7. A source with unresolved modification/adaptation rights should not be enabled for derived-database ingestion merely because redistribution is allowed.

## DATA.GOV.HK boundary

The reviewed portal terms allow important reuse activities but the reviewed wording did not clearly grant general modification/adaptation rights for derived terminology databases. Automated ingestion therefore remains disabled until an individually reviewed dataset has a narrower machine-readable approval.

## Change control

When a source decision changes:

1. update the source review evidence;
2. update `sources.yaml` including any exact resource IDs;
3. update current source/review summaries if material;
4. record durable architecture/policy decisions under `docs/adr/`;
5. update `PROJECT_STATE.md` only when project state changes materially.
