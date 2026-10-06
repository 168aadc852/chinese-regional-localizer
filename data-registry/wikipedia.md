# Chinese Wikipedia

Status: **approved_with_conditions**

## Intended use
Regional proper names, article-level conversion rules, shared conversion groups, redirects/aliases and contextual evidence for CN / HK / TW localization.

## Region coverage
CN / HK / TW and other Chinese variants

## Official source
URL: https://zh.wikipedia.org/
Publisher: Wikimedia Foundation / Chinese Wikipedia community

## Licence review
Primary text licence: Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0). Some text may also be available under GFDL depending on contribution history, but this project should standardize its reuse path on CC BY-SA 4.0 unless a specific reason requires otherwise.

Licence evidence URLs:
- https://foundation.wikimedia.org/wiki/Terms_of_Use
- https://foundation.wikimedia.org/wiki/Legal:Text_of_the_Creative_Commons_Attribution-ShareAlike_4.0_International_License/en
- https://foundation.wikimedia.org/wiki/Legal:Wikimedia_Developer_App_Guidelines/en

- Commercial use: **Yes**, subject to licence conditions.
- Modification: **Yes**.
- Redistribution: **Yes**, subject to attribution and share-alike obligations.
- Attribution: **Required** for reused text/material. Wikimedia's Terms permit attribution through a link/URL to the reused page (whose history records contributors), a qualifying stable copy, or a list of authors. For this project, store source page URL/title plus revision ID or extraction timestamp where practical.
- Share-alike / copyleft: **Yes.** Adapted material based on CC BY-SA content must be distributed under CC BY-SA 4.0 or a compatible licence.
- Database-specific obligations: CC BY-SA 4.0 includes sui generis database-right provisions. If all or a substantial portion of licensed database contents is incorporated into a database in which sui generis database rights arise, the database may be treated as adapted material for ShareAlike purposes. Treat a substantial Wikipedia-derived terminology collection conservatively as a separate CC BY-SA data pack.
- API/download/scraping restrictions: Prefer official Wikimedia dumps and documented MediaWiki APIs. Do not rely on scraping rendered pages as the primary ingestion method. Operational API limits and Wikimedia terms must be respected independently of the content licence.

## Packaging decision
**Approved with conditions, but NOT for the permissive/core data layer.**

Create a separate Wikipedia-derived data package, for example:

`regional_wikipedia.sqlite`

Recommended contents may include:
- NoteTA / article-level regional conversion rules
- public/shared conversion groups
- regional proper-name mappings
- redirects and aliases where useful
- source page/revision provenance

Recommended distribution treatment:
1. Mark the Wikipedia-derived package as CC BY-SA 4.0.
2. Include the CC BY-SA 4.0 licence notice / link and attribution information.
3. Preserve source page identity and revision/provenance metadata so users can trace records back to contributors/history.
4. Mark transformations / normalization performed by this project.
5. Do not merge Wikipedia-derived records into a dataset that the project represents as Apache-2.0, MIT or CC0.

## Update method
Preferred ingestion methods:
1. Wikimedia official Chinese Wikipedia dumps for periodic full rebuilds.
2. MediaWiki RecentChanges / revision APIs for incremental refreshes where appropriate.
3. Store page ID/title, revision ID and extraction timestamp for records used in releases.

A periodic full rebuild should be used to recover safely if incremental history is missed or the synchronizer has been offline too long.

## Important scope note
Facts as such may not always be copyrightable, and Wikimedia's Terms note that contributed facts may in some cases be freely reusable. However, this project should not try to make record-by-record legal originality determinations for NoteTA rules, conversion groups or curated mappings. The conservative project rule is to treat the extracted Wikipedia-derived pack as CC BY-SA unless a record is independently sourced from a more permissive source such as Wikidata CC0.

## Review evidence / rationale
Wikimedia's current Terms of Use and developer guidance state that Wikipedia text is reusable under CC BY-SA 4.0, including commercial reuse, provided attribution and ShareAlike requirements are followed. CC BY-SA 4.0 expressly permits extracting/reusing database contents where its licensed database rights apply, while imposing attribution and ShareAlike conditions for covered reuse.

This makes Chinese Wikipedia usable for the project, but it should remain an isolated share-alike data layer rather than contaminating the permissive/core dataset.

Last reviewed: 2026-10-06
Review status: formal first-pass review complete; specific MediaWiki modules/templates should still be checked if executable code rather than text/data is later redistributed.
