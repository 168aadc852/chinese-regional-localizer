# OpenHowNet

Status: **pending_review**

## Intended use
Chinese word senses, sememes, semantic similarity, ambiguity detection and context-aware candidate ranking.

## Official source
Repository: https://github.com/thunlp/OpenHowNet
Project site/download: https://openhownet.thunlp.org/
Publisher: THUNLP / OpenHowNet

## Evidence reviewed
- The GitHub repository is published under an **MIT License**.
- The Python package metadata classifies the software as MIT licensed.
- The repository README describes the project as “Core Data of HowNet and OpenHowNet Python API”.
- However, the actual HowNet core data is not stored as an ordinary repository file in the current repo; the API instructs users to run `OpenHowNet.download()` or download the HowNet dictionary separately from the project website.
- The root MIT licence text uses the standard wording referring to “software and associated documentation files”.

## Rights review
### API / repository code
- Commercial use: **Yes under MIT**
- Modification: **Yes**
- Redistribution: **Yes with MIT notice**

### Downloaded HowNet core data
- Commercial use: **Not yet independently confirmed from a data-specific licence**
- Modification: **Not yet independently confirmed**
- Redistribution: **Not yet independently confirmed**
- Attribution/citation: Project requests citation if data/API are used in research, but citation guidance is not a substitute for an explicit redistribution licence

## Packaging decision
Do **not** redistribute the downloaded HowNet core data in this project yet.

Allowed for now:
- use the MIT-licensed OpenHowNet API/code if useful;
- treat online/local HowNet results as reference during research where permitted;
- keep the core data out of release database packs until a data-specific licence or authoritative statement confirms redistribution/modification rights.

## Next verification task
Obtain an authoritative statement from OpenHowNet/THUNLP that explicitly says whether the downloadable HowNet core data itself is covered by MIT or another redistribution licence.

Last reviewed: 2026-10-06
