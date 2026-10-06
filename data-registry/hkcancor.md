# Hong Kong Cantonese Corpus (HKCanCor)

Status: **approved_with_conditions**

## Intended use
Hong Kong Cantonese vocabulary extraction, word-frequency evidence, segmentation, Jyutping and part-of-speech support.

## Official source
Repository: https://github.com/fcbond/hkcancor
Publisher/authors: Hong Kong Cantonese Corpus project; K. K. Luke and May L. Y. Wong, repository maintained/distributed by project contributors

## Licence review
The official repository states that the corpus is released under **Creative Commons Attribution 4.0 International (CC BY 4.0)**.

The corpus is word-segmented and annotated with pronunciation and POS. Full transcriptions are downloadable; complete audio is not released (only samples are available).

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Indicate changes: **Required under CC BY 4.0 when sharing adapted material**
- ShareAlike: **No**

## Packaging decision
Approved for an attribution-required Hong Kong/Cantonese evidence layer.

Recommended project use:
- derive HK Cantonese word-frequency/statistical signals;
- validate tokenisation and POS;
- extract candidate Cantonese lexical items;
- compare candidate terms against other HK resources.

If distributing derived frequency lists or lexical extracts, retain corpus attribution and document the derivation method.

Do not bundle audio unless separately reviewed; the project does not need audio for the initial localisation tool.

## Update method
Use official repository corpus files and record commit/version. Preserve citation recommended by the project.

Last reviewed: 2026-10-06
