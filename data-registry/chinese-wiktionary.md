# Chinese Wiktionary

Status: **approved_with_conditions**

## Intended use
Definitions, lexical senses, synonyms/antonyms, pronunciation, etymology, regional usage labels, translations and aliases.

## Official source
Website: https://zh.wiktionary.org/
Copyright information: https://zh.wiktionary.org/wiki/Wiktionary:%E7%89%88%E6%9D%83%E4%BF%A1%E6%81%AF
CC BY-SA 4.0 text: https://zh.wiktionary.org/wiki/Wiktionary:CC_BY-SA_4.0%E5%8D%94%E8%AD%B0%E6%96%87%E6%9C%AC
Publisher: Wikimedia community / Wikimedia Foundation platform

## Licence review
Chinese Wiktionary states that its textual content is distributed under **Creative Commons Attribution-ShareAlike 4.0** and the **GNU Free Documentation License**.

For project reuse, use the CC BY-SA 4.0 route for extracted textual/lexical content unless a specific item has different licensing.

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Indicate changes: **Required for adapted material**
- Share-alike/copyleft: **Yes** for adaptations
- Database rights: CC BY-SA 4.0 includes rules for extraction/reuse of substantial database contents where applicable

## Packaging decision
Do not merge Chinese Wiktionary-derived content into the permissive core database.

Use a separate ShareAlike lexical/semantic pack with per-record provenance sufficient to satisfy attribution, ideally including:
- source page title;
- page URL;
- revision ID or revision timestamp;
- extraction timestamp;
- licence class;
- indication of transformations/normalisation.

## Scope boundary
This review covers **textual Wiktionary content** under the project’s stated text licences.

Do not assume that images, audio, stroke-order media or other Commons-hosted files carry the same licence; those assets must be checked individually or excluded.

## Update method
Prefer official Wikimedia dumps/API/revision data over scraping rendered pages. Support incremental refresh while preserving revision-level provenance.

Last reviewed: 2026-10-06
