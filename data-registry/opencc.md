# OpenCC

Status: **approved_with_conditions**

## Intended use

Character/script conversion and regional terminology rules for the reviewed CN → Hant → HK/TW forward pipeline.

## Official source

Repository: https://github.com/BYVoid/OpenCC  
Publisher/maintainer: OpenCC project

## Licence review

Licence: **Apache-2.0** for reviewed project/dictionary files.

- Commercial use: yes.
- Modification: yes.
- Redistribution: yes, subject to Apache-2.0 conditions/notices.
- Attribution/notices: required where applicable.
- Share-alike: no.

Exact reviewed production commit:

`3ac34aa439a9908dd49fa92b5174b46314787ac2`

## Machine-readable ingest scope

The importer does not treat broad project approval as permission for every file. `sources.yaml` explicitly allows these resources:

- `opencc:STPhrases.txt`
- `opencc:STCharacters.txt`
- `opencc:HKPhrases.txt`
- `opencc:HKVariantsPhrases.txt`
- `opencc:HKVariants.txt`
- `opencc:TWPhrases.txt`
- `opencc:TWVariantsPhrases.txt`
- `opencc:TWVariants.txt`

Anything else requires a manifest/review update first.

## Update behavior

Each dictionary is an independent resource snapshot with commit, raw URL and SHA-256 provenance. Repeat import of identical bytes/revision is idempotent. A new snapshot supersedes/deactivates the older runtime rules while retaining history.

## Packaging

Core/permissive pack, while retaining Apache-2.0 obligations/notices. Re-check notices before the first public binary/data release.

## Runtime limitation

The project preserves reviewed staged dictionary/candidate semantics but does not claim full OpenCC runtime parity. Generated phrase resources, complete mmseg behavior, reverse routes and all configs/plugins remain outside current scope.

Last reviewed: 2026-10-07
