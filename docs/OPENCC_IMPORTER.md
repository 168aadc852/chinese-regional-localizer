# OpenCC Dictionary Importer — Phase 2C

The importer supplies the reference engine's script/regional conversion rules while preserving OpenCC stages, dictionary precedence, candidate ordering and provenance.

## Pinned upstream

- repository: `BYVoid/OpenCC`;
- commit: `3ac34aa439a9908dd49fa92b5174b46314787ac2`;
- licence: Apache-2.0.

## Reviewed resources

Script stage:
- `STPhrases.txt`;
- `STCharacters.txt`.

Hong Kong stage:
- `HKPhrases.txt`;
- `HKVariantsPhrases.txt`;
- `HKVariants.txt`.

Taiwan stage:
- `TWPhrases.txt`;
- `TWVariantsPhrases.txt`;
- `TWVariants.txt`.

Each file has an exact `opencc:<filename>` allow-list identifier in `sources.yaml`; an unlisted file is rejected even though the broader OpenCC source is approved.

## Update semantics

Each dictionary is a separate `resource_key`. Re-importing identical revision + checksum is idempotent and does not add duplicate source versions/rules. A changed snapshot:

1. deactivates the old resource's rules;
2. marks its source version non-current;
3. inserts the new version/rules;
4. preserves historical rows for auditability.

Legacy pre-v0.2 source-version rows are superseded on the first refresh.

## Dictionary precedence

Base priorities retain the reviewed short-circuit order:

- ST phrase `300000`, ST character `200000`;
- HK/TW regional phrase `500000`;
- variant phrase `400000`;
- variant character `300000`.

Candidate rank is subtracted within a dictionary. Longest-source matching remains an engine concern, not a priority trick.

## Provenance

Every dictionary version stores the pinned commit, exact raw URL, SHA-256, resource key, stage/locale notes and retrieval time. Every rule retains dictionary line ID and candidate metadata.

## Scope limitation

This is OpenCC-compatible reference behavior, not full runtime parity. It does not yet model generated regional segmentation dictionaries, all mmseg/union behavior, reverse routes or all OpenCC configs/plugins.
