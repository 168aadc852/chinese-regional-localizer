# OpenCC Phrase Importer — Phase 1B

This importer is the project's first real regional-term conversion source.

## Pinned upstream

- Repository: `BYVoid/OpenCC`
- Commit: `3ac34aa439a9908dd49fa92b5174b46314787ac2`
- Licence: Apache-2.0

Initial imported files:

- `STPhrases.txt` — `zh-CN -> zh-Hant`
- `HKPhrases.txt` — `zh-Hant -> zh-HK`
- `TWPhrases.txt` — `zh-Hant -> zh-TW`

The upstream OpenCC configs show that regional conversion is staged. For example, Simplified-to-Hong-Kong-with-phrases first applies Simplified/Traditional conversion and then Hong Kong phrase/variant conversion. Taiwan-with-phrases follows the same staged pattern.

## Why the database stores stages

The importer deliberately does **not** flatten these files into one universal replacement table.

A future `zh-CN -> zh-HK` conversion can compose:

1. `zh-CN -> zh-Hant` rules from the script-conversion stage;
2. `zh-Hant -> zh-HK` rules from the Hong Kong regional stage.

Likewise, `zh-CN -> zh-TW` can compose the script stage with the Taiwan regional stage.

This preserves the source model and avoids falsely claiming that every regional phrase is a direct Simplified-Chinese mapping.

## Multiple target candidates

OpenCC allows a source key to map to several candidate values separated by spaces, for example a first/default candidate followed by alternatives.

The importer creates one `term_rules` row per candidate and preserves the candidate order in:

- `priority` — earlier candidates receive higher priority;
- `context_constraint` JSON — includes `candidate_rank`, `candidate_count`, source dictionary and stage.

The project must not discard lower-ranked candidates. Later context/ranking logic may use them when the default choice is unsuitable.

## Provenance

Each imported dictionary gets a separate `source_versions` row containing:

- pinned OpenCC commit;
- upstream raw URL;
- SHA-256 of the exact imported bytes;
- dictionary filename and locale-stage notes.

Each `term_rules` row records:

- `source_id = opencc`;
- the dictionary-specific `source_version_id`;
- `upstream_record_id = <filename>:<line>`;
- pinned upstream URL.

## Parser safety

The parser requires the reviewed OpenCC header markers before accepting data rows:

- OpenCC dictionary marker;
- exact `# File:` name;
- documented tab-separated format marker;
- Apache-2.0 licence marker.

Blank lines and comments are ignored. Data rows must contain exactly one tab separator, a non-empty source key and at least one target candidate.

## Scope limitation

Phase 1B imports only phrase dictionaries. It does **not** yet reproduce the complete OpenCC runtime, which also uses character dictionaries, variant dictionaries, generated segmentation dictionaries, maximum-matching segmentation and short-circuit/union match policies.

Future work should add those resources explicitly rather than pretending the current importer is a drop-in reimplementation of OpenCC.

## Local usage

Fixture/offline test build:

```bash
python scripts/import_opencc_dictionaries.py \
  --fixture-dir data/fixtures/opencc \
  --db build/opencc.sqlite \
  --reset
```

Explicit network download of the pinned three files:

```bash
python scripts/import_opencc_dictionaries.py \
  --download \
  --db build/opencc.sqlite \
  --reset
```

Normal application runtime remains offline-first; network access here is a database-build/update operation only.
