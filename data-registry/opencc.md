# OpenCC

Status: **approved_with_conditions**

## Intended use
Character/script conversion and regional terminology rules, including CN / HK / TW variant conversion and selected regional phrase mappings.

## Region coverage
CN / HK / TW and related Chinese variants

## Official source
URL: https://github.com/BYVoid/OpenCC
Publisher / maintainer: OpenCC project (BYVoid/OpenCC)

## Licence review
Licence name: Apache License 2.0
Licence evidence URLs:
- https://github.com/BYVoid/OpenCC/blob/master/LICENSE
- https://github.com/BYVoid/OpenCC/blob/master/README.md
- Example dictionary licence declaration: https://github.com/BYVoid/OpenCC/blob/master/data/dictionary/STCharacters.txt

- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution / notices: **Required when redistributing**. Include a copy of Apache-2.0, retain applicable copyright / patent / trademark / attribution notices, and mark modified files prominently. If an upstream distribution includes a NOTICE file, preserve the relevant NOTICE contents as required by Apache-2.0 section 4(d).
- Share-alike / copyleft: **No**
- Database-specific obligations: **None identified beyond the Apache-2.0 terms applying to the distributed OpenCC dictionary/configuration files.** Current dictionary files explicitly identify themselves as Apache-2.0.
- API/download/scraping restrictions: **None identified for using the published open-source repository and releases.** Prefer release/download artifacts or a pinned upstream revision rather than scraping rendered GitHub pages.

## Packaging decision
**Approved with conditions.**

OpenCC code, configuration files and dictionary files that explicitly fall under Apache-2.0 may be bundled in the application or in the permissive/core data layer, provided the Apache-2.0 redistribution requirements are met.

Recommended packaging rules:
1. Record the exact OpenCC version or commit used.
2. Bundle or otherwise provide the Apache-2.0 licence text with redistributed OpenCC material.
3. Preserve relevant upstream notices and file headers.
4. Clearly mark any modified OpenCC dictionary/configuration files as modified.
5. Re-check third-party components separately if they are bundled directly rather than merely used as build/runtime dependencies.

## Phase 1B importer baseline

The first real OpenCC importer is pinned to commit:

`3ac34aa439a9908dd49fa92b5174b46314787ac2`

Initial reviewed/imported phrase files:

- `STPhrases.txt` — script-stage `zh-CN -> zh-Hant`
- `HKPhrases.txt` — regional-stage `zh-Hant -> zh-HK`
- `TWPhrases.txt` — regional-stage `zh-Hant -> zh-TW`

The importer intentionally preserves OpenCC's staged model and multiple target candidates. It does not claim to reproduce the full OpenCC runtime segmentation/match-policy behaviour yet. See `docs/OPENCC_IMPORTER.md`.

Implementation:

- `scripts/import_opencc_dictionaries.py`
- `tests/test_opencc_importer.py`
- `data/fixtures/opencc/`

Each imported dictionary records its own SHA-256 and `source_versions` provenance.

## Update method
Prefer pinned OpenCC releases / tags or a recorded upstream commit. A database-builder job may periodically check the official BYVoid/OpenCC repository for a newer release and rebuild the local conversion resources after tests pass.

## Review evidence / rationale
The official OpenCC README identifies the project licence as Apache License 2.0. The repository's Apache-2.0 LICENSE grants reproduction, modification and redistribution rights subject to section 4 conditions. OpenCC dictionary files such as `STCharacters.txt`, `HKVariants.txt`, `TWVariants.txt` and `TSPhrases.txt` explicitly state `License: Apache-2.0 (see LICENSE)`, which is important because this project intends to redistribute dictionary data, not only link to the conversion library.

Last reviewed: 2026-10-06
Review status: formal first-pass review complete; Phase 1B phrase importer implemented and CI-tested; re-check before first public binary/data release.
