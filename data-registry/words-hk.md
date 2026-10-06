# Words.hk

Status: **approved_with_conditions**

## Intended use
Hong Kong/Cantonese lexical signals, public-domain word lists/pronunciations, and reference-only use of the full dictionary where necessary.

## Region coverage
Hong Kong / Cantonese

## Official sources
Public-domain word list: https://words.hk/faiman/analysis/wordslist/
Open data licence: https://words.hk/base/hoifong/
Dictionary data download: https://words.hk/faiman/request_data/
Publisher: Hong Kong Lexicography Limited / Words.hk

## Licence review
Words.hk has multiple licensing layers and they must not be mixed.

### A. Public-domain word list and pronunciations
The official word-list page explicitly states:
- “Data License: public domain” / “授權：公有領域”;
- the dataset contains dictionary entry words and pronunciations;
- CSV and JSON downloads are provided.

For this specific public-domain dataset:
- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Not legally required based on the public-domain statement; credit is appreciated**
- Share-alike/copyleft: **No**

### B. Full dictionary/open-data content
The full dictionary data is released under the project’s **Non-Commercial Open Data License 1.0** where so marked.

That licence permits copying, modification, adaptation, translation and redistribution, but generally **prohibits commercial use** unless an exception or separate agreement applies. It also imposes notice/credit/link conditions and prohibits sublicensing.

For the full dictionary:
- Commercial use: **No, by default**
- Modification: **Yes, subject to the licence**
- Redistribution: **Yes, subject to licence conditions**
- Attribution/notices: **Required**
- Sublicensing: **Not permitted**

## Packaging decision
Approved for the permissive/core candidate layer **only for the explicitly public-domain word-list/pronunciation dataset**.

Do not package the full dictionary definitions/examples in a commercial-capable open-source distribution unless a separate compatible licence is obtained.

Recommended architecture:
- `words-hk-public-domain` → permitted core lexical signal source;
- full Words.hk dictionary → `reference_only` for this project unless separately licensed.

## Update method
Use the official CSV/JSON public-domain word-list exports and record download/update date. Do not scrape other dictionary content and assume it shares the same public-domain status.

Last reviewed: 2026-10-06
