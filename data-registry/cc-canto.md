# CC-Canto

Status: **approved_with_conditions**

## Intended use
Cantonese vocabulary, Jyutping readings, Traditional/Simplified forms and Cantonese-English semantic support.

## Region coverage
Hong Kong / Cantonese

## Official source
Website: https://www.cc-canto.org/
Download/licence page: https://www.cc-canto.org/download.html
Publisher: CC-Canto / Pleco Software Incorporated

## Licence review
The official download page states that CC-Canto and the Cantonese readings for CC-CEDICT are distributed under **Creative Commons Attribution-ShareAlike 3.0**.

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Share-alike/copyleft: **Yes** for adapted material
- Format: distributed in CC-CEDICT-style format with Jyutping readings

## Packaging decision
Do not merge CC-Canto-derived content into the permissive core database.

Use a separate ShareAlike-compatible Cantonese lexical pack with:
- source attribution;
- licence notice/link;
- source version/download date;
- indication of modifications;
- separation from CC0 / Apache / permissive datasets.

If CC-CEDICT content and CC-Canto content are combined, record the licence/provenance of each component and ensure the resulting pack satisfies the applicable ShareAlike obligations.

## Update method
Use the official downloadable CC-Canto and Cantonese-reading files. Do not scrape website search results when official downloads are available.

Last reviewed: 2026-10-06
