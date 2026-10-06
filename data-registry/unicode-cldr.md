# Unicode / Unihan / CLDR

Status: **approved_with_conditions**

## Intended use
Character variants, Unicode properties, Han/Unihan data and locale conventions for `zh-CN`, `zh-HK`, `zh-TW` and related locales.

## Region coverage
Unicode and Chinese locales

## Official sources
Unicode licensing policy: https://unicode.org/policies/licensing_policy.html
Unicode Terms of Use: https://unicode.org/copyright.html
Unicode licence FAQ: https://www.unicode.org/faq/unicode_license.html
CLDR releases: https://cldr.unicode.org/index/downloads
Publisher: Unicode Consortium

## Licence review
Most Unicode **Data Files** and software are released under the **Unicode License v3 (SPDX: Unicode-3.0)**, an OSI-approved, highly permissive licence based on MIT and expressly covering data/data files.

Unicode Terms of Use define Data Files to include computer data files under official Unicode Public directories and Unicode GitHub organisation, unless a specific file has another licence/restriction.

CLDR release notes and download directories point to the Unicode Terms of Use/licence, and current CLDR data is distributed with licence files.

- Commercial use: **Yes for licensed Data Files/software**
- Modification: **Yes for Unicode-licensed Data Files/software**
- Redistribution: **Yes, subject to Unicode License v3 and file-specific notices**
- Attribution/licence notice: **Retain the applicable Unicode copyright/licence notice**
- Share-alike/copyleft: **No**
- Database-specific obligations: None analogous to ODbL for ordinary Unicode-3.0 data files

## Important exclusions / boundaries
Do not treat every item on unicode.org as a redistributable data file.

The broader website Terms of Use distinguish:
- Data Files/software → generally Unicode License v3;
- standards, technical reports, code charts, publications and fonts → may have different or more restrictive permissions.

For this project, only ingest files that are clearly Unicode Data Files or CLDR release data covered by Unicode-3.0, unless separately reviewed.

## Packaging decision
Approved for core/permissive use **only for clearly identified Unicode-3.0 Data Files**, such as relevant Unihan/Unicode data and CLDR machine-readable data.

Store:
- exact file/release version;
- upstream path;
- licence notice;
- any file-specific exception.

Do not package Unicode code-chart PDFs, fonts, or standards prose as though they were ordinary Unicode-3.0 data files.

## Update method
Use versioned Unicode/CLDR releases rather than unversioned scraping. CLDR publishes stable numbered releases; record release number and checksums when practical.

Last reviewed: 2026-10-06
