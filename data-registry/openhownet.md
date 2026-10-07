# OpenHowNet

Status: **approved_with_conditions**

## Intended use
Chinese word senses, sememes, semantic similarity, ambiguity detection and context-aware candidate ranking.

## Official sources
Repository: https://github.com/thunlp/OpenHowNet
Project/download page: https://openhownet.thunlp.org/download
Publisher: THUNLP / OpenHowNet

## Evidence reviewed
- The official OpenHowNet download page directly offers the HowNet core data download.
- On that same download page, the section titled “開源協議與引用規範” states that OpenHowNet is based on the **MIT licence**.
- The same official section explicitly discusses use of OpenHowNet-provided **data or API** when requesting academic citation, linking the licence/citation statement to both data and API rather than the Python package alone.
- The official GitHub repository carries the standard MIT License, copyright THUNLP 2019.
- The repository README identifies the separately downloadable HowNet dictionary as “HowNet core data”.

## Rights review
For the OpenHowNet-provided downloadable HowNet core data covered by the official OpenHowNet download/licence statement:
- Commercial use: **Yes under MIT**
- Modification / adaptation: **Yes**
- Redistribution: **Yes**
- Attribution / notice: **Preserve the MIT copyright and permission notice**
- Share-alike/copyleft: **No**
- Research citation: **Requested by the project; preserve citation/provenance metadata where practical**

## Packaging decision
The OpenHowNet-provided core-data download may be used in a permissive/core-compatible pack subject to the MIT notice requirement and exact-resource provenance.

Approval is intentionally narrow:
- only use data distributed through the official OpenHowNet download mechanism or an exact resource whose OpenHowNet MIT coverage is documented;
- preserve source URL, retrieval date, version/checksum where available, and MIT notice;
- do not assume unrelated historical/proprietary HowNet distributions are covered merely because they contain similar data;
- production ingestion remains disabled until the exact downloadable resource is pinned in `ingest_resources` and importer validation is designed.

## Update method
Prefer the official OpenHowNet core-data download. Record the exact download URL/resource identity, retrieval date, checksum and any version metadata available at ingestion time.

Last reviewed: 2026-10-07
