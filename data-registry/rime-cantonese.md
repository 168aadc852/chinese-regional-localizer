# Rime Cantonese

Status: **approved_with_conditions**

## Intended use
Cantonese/Hong Kong lexical data, pronunciation/Jyutping support and related input-method vocabulary.

## Region coverage
Hong Kong / Cantonese

## Official source
URL: https://github.com/rime/rime-cantonese
Maintainer: Cantonese Computational Linguistics Infrastructure Development Workgroup (CanCLID)

## Licence review
The official README states:
- the **main part** of the work is licensed under **Creative Commons Attribution 4.0 International (CC BY 4.0)**;
- `jyut6ping3.maps` is licensed under **Open Data Commons Open Database License 1.0 (ODbL 1.0)**.

### Main Rime Cantonese content — CC BY 4.0
- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Indicate changes: **Required**
- Share-alike: **No**

### `jyut6ping3.maps` — ODbL 1.0
- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Share-alike/database obligations: **Yes** for publicly used derivative databases under ODbL rules
- Source note: the file itself states that it contains OpenStreetMap-derived data and is ODbL-licensed

## Packaging decision
Approved with strict file-level separation.

Recommended handling:
- CC BY 4.0 main lexical/pronunciation content may be used in an attribution-required pack;
- exclude `jyut6ping3.maps` from the permissive/core pack;
- if map-derived data is needed, place it in a separate ODbL-compatible database/module with ODbL notices and source availability requirements.

Do not collapse all repository files under one licence label.

## Update method
Track official GitHub releases/commits. Record the exact source file path and licence class for every imported file.

Last reviewed: 2026-10-06
