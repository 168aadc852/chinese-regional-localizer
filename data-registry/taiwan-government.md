# Taiwan Government and terminology open data

Status: **approved_with_conditions**

## Intended use
Official/professional terminology, especially TW/CN terminology comparison datasets where each dataset explicitly states an open licence compatible with project redistribution.

## Region coverage
Taiwan; selected cross-strait datasets

## Official sources reviewed
Government Open Data Platform: https://data.gov.tw/
Licence: https://data.gov.tw/license

Verified National Academy for Educational Research (國家教育研究院) terminology examples:
- Management terminology: https://data.gov.tw/dataset/15440
- Cross-strait computer terminology: https://data.gov.tw/dataset/15275
- Cross-strait mathematics terminology: https://data.gov.tw/dataset/15290
- Cross-strait primary/secondary-school information terminology: https://data.gov.tw/dataset/15406
- Cross-strait Chinese medicine terminology: https://data.gov.tw/dataset/15269

Publisher / data provider for the verified examples: 國家教育研究院 (National Academy for Educational Research, Taiwan)

## Licence review
Licence name: 政府資料開放授權條款－第1版 / Open Government Data License, version 1.0 (Taiwan)
Licence evidence URL: https://data.gov.tw/license

The verified datasets above explicitly state this licence on their dataset pages.

- Commercial use: **Yes.** The licence grants use for any purpose and expressly permits derivative products/services.
- Modification: **Yes.** Reproduction, compilation, adaptation and other modification are permitted.
- Redistribution: **Yes.** Distribution and public transmission are permitted.
- Sublicensing: **Yes.** The licence expressly permits sublicensing.
- Attribution: **Required.** Use of the open data and derivative works must include an attribution statement identifying the original data-providing organisation and dataset. Failure to provide the required attribution is treated by the licence as failure to obtain the licence.
- Share-alike / copyleft: **No general ShareAlike requirement identified.** The licence is stated to be compatible with CC BY 4.0.
- Database-specific obligations: Attribution applies to the open dataset and derivatives. Patent and trademark rights are excluded from the grant. Third-party moral rights / privacy / other rights are not automatically cleared by the licence.
- API/download/scraping restrictions: Use the official dataset download resources (for example CSV) rather than scraping rendered pages. Dataset-specific metadata, availability and update frequency should be recorded.

## Packaging decision
**Approved with conditions ONLY for individually verified datasets whose own metadata explicitly states the Open Government Data License v1.0 or another compatible licence.**

Do **not** treat all Taiwan government websites or all datasets on `data.gov.tw` as automatically approved.

Recommended packaging treatment:
1. Store each imported Taiwan government dataset with its dataset ID, official URL, provider and licence version.
2. Include attribution in `NOTICE` / data acknowledgements and in machine-readable provenance metadata.
3. Preserve dataset/update timestamps so releases can be reproduced.
4. If a future dataset has a different licence, review it separately before ingestion.

## Recommended first datasets for this project
The verified NAER datasets are especially valuable because their fields include combinations such as:
- English name
- Taiwan Chinese name
- Mainland China translation
- source website

These can provide authoritative CN ↔ TW professional terminology mappings in domains such as computing, mathematics, management, medicine and education.

## Update method
Use official dataset resource/download URLs and metadata from `data.gov.tw`. Respect each dataset's declared update frequency (many reviewed NAER terminology datasets are marked as updated irregularly / 不定期更新).

A future importer should keep a manifest containing at least:
- dataset ID
- dataset title
- provider
- source URL
- licence/version
- retrieved_at
- upstream metadata updated time
- content checksum

## Review evidence / rationale
The Taiwan Open Government Data License v1.0 grants worldwide, royalty-free, non-exclusive, irrevocable rights to reproduce, distribute, transmit, compile and adapt data for any purpose, and permits sublicensing. It imposes a clear attribution requirement and states compatibility with Creative Commons Attribution 4.0 International.

Multiple National Academy for Educational Research terminology datasets have been individually verified to declare this licence and to expose Taiwan Chinese / Mainland China terminology fields directly relevant to this project.

Last reviewed: 2026-10-06
Review status: formal first-pass review complete for the licence framework and the example datasets listed above; every additional dataset must still be individually verified before ingestion.
