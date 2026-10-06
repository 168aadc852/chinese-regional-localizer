# OpenCC Dictionary Importer — Phases 1B / 2B

This importer provides the first real script and regional-terminology conversion layer.

## Pinned upstream

- Repository: `BYVoid/OpenCC`
- Commit: `3ac34aa439a9908dd49fa92b5174b46314787ac2`
- Licence: Apache-2.0

## Imported resources

### Script stage — `zh-CN -> zh-Hant`

1. `STPhrases.txt` — phrase mappings
2. `STCharacters.txt` — character fallback

### Hong Kong stage — `zh-Hant -> zh-HK`

1. `HKPhrases.txt` — regional vocabulary/proper-name phrases
2. `HKVariantsPhrases.txt` — phrase-level variant exceptions
3. `HKVariants.txt` — character variants

### Taiwan stage — `zh-Hant -> zh-TW`

1. `TWPhrases.txt` — regional vocabulary/proper-name phrases
2. `TWVariantsPhrases.txt` — phrase-level variant exceptions
3. `TWVariants.txt` — character variants

The order above reflects the reviewed OpenCC short-circuit configuration semantics.

## Why stages remain separate

The importer deliberately does **not** flatten all dictionaries into one universal replacement table.

For `zh-CN -> zh-HK`, the reference engine composes:

1. `zh-CN -> zh-Hant`;
2. `zh-Hant -> zh-HK`.

For `zh-CN -> zh-TW`, it composes:

1. `zh-CN -> zh-Hant`;
2. `zh-Hant -> zh-TW`.

This preserves the source model and makes provenance/explanations meaningful.

## Dictionary precedence

Phase 2B adds dictionary-level base priorities matching reviewed OpenCC short-circuit order.

Script stage:

- ST phrase: `300000`
- ST character: `200000`

HK stage:

- HK regional phrase: `500000`
- HK variant phrase exception: `400000`
- HK character variant: `300000`

TW stage:

- TW regional phrase: `500000`
- TW variant phrase exception: `400000`
- TW character variant: `300000`

Candidate rank is then subtracted from the dictionary base priority.

This keeps two distinct concepts separate:

- **longest source phrase matching** is handled by the localization engine;
- **dictionary/candidate precedence for the same matched source string** is represented by priority.

## Phrase exceptions

OpenCC explicitly uses phrase-exception dictionaries to protect phrases from undesirable character-level conversion.

The reference engine's longest-match scan naturally preserves this behavior. Example fixture:

```text
張棟樑 -> 張棟樑
```

wins as a complete Taiwan phrase exception instead of later allowing the character rule:

```text
樑 -> 梁 / 樑
```

to modify the name.

## Multiple target candidates

OpenCC permits a key to map to several candidates separated by spaces.

The importer stores one `term_rules` row per candidate and preserves:

- candidate order;
- dictionary base priority;
- candidate rank/count;
- source dictionary and stage.

Lower candidates are not discarded. Phase 2A currently chooses the highest-priority candidate when unambiguous, while exposing alternatives in explanation output.

## Provenance

Each imported dictionary gets a separate `source_versions` row containing:

- pinned OpenCC commit;
- upstream raw URL;
- SHA-256 of exact imported bytes;
- dictionary filename, stage, locale route and dictionary base priority.

Each rule records:

- `source_id = opencc`;
- dictionary-specific `source_version_id`;
- `upstream_record_id = <filename>:<line>`;
- upstream URL;
- rule type and priority;
- context JSON describing source dictionary/stage/candidate ranking.

## Parser safety

The parser requires reviewed header markers before accepting data:

- OpenCC dictionary marker;
- exact `# File:` name;
- tab-separated format marker;
- Apache-2.0 licence marker.

Blank lines/comments are ignored. Data rows require exactly one tab separator, non-empty source key and at least one target candidate.

## Current tested examples

The fixture/regression suite includes:

- `布拉德·皮特 -> 畢·彼特` for HK;
- `人工智能 -> 人工智慧` for TW;
- `一见钟情 -> 一見鍾情` through the script phrase stage;
- `见 -> 見` through character fallback;
- `檯 -> 枱` for HK variant conversion;
- `爲 -> 為` for TW variant conversion;
- `張棟樑` phrase-exception protection;
- identity and multi-candidate mappings.

## Scope limitation

This is substantially broader than Phase 1B but is still **not a complete reimplementation of the OpenCC runtime**.

Not yet modelled completely:

- generated regional segmentation dictionaries such as build-generated ST phrase resources;
- all OpenCC segmentation/match-policy implementation details;
- reverse HK/TW/CN directions;
- all available OpenCC configs/plugins.

Those should be added explicitly and tested rather than claimed implicitly.

## Local usage

Offline fixture build:

```bash
python scripts/import_opencc_dictionaries.py \
  --fixture-dir data/fixtures/opencc \
  --db build/opencc.sqlite \
  --reset
```

Explicit network build from all reviewed commit-pinned files:

```bash
python scripts/import_opencc_dictionaries.py \
  --download \
  --db build/opencc.sqlite \
  --reset
```

Normal application localization remains offline-first; network access here is only for explicit database build/update work.
