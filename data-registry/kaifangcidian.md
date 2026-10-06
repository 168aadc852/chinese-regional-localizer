# Kaifangcidian / Open Chinese Dictionary

Status: **approved_with_conditions**

## Intended use
Open Chinese pronunciation/dictionary data where individual repositories carry compatible licences.

## Project family
Organisation: https://github.com/kfcd/
Reviewed dataset example: https://github.com/kfcd/hyzd

## Licence review
The reviewed `kfcd/hyzd` repository (開放漢語字典 / 現代漢語字音數據庫) explicitly states that the data in that repository is released under **Creative Commons Attribution 3.0 Unported (CC BY 3.0)**.

For the reviewed `hyzd` dataset:
- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- ShareAlike: **No**

## Packaging decision
Approved **only on a per-repository/per-dataset basis**.

Current approved subset:
- `kfcd/hyzd` → attribution-required data layer under CC BY 3.0.

Do not infer that every repository under the `kfcd` organisation carries the same licence. Any additional Kaifangcidian dataset must have its own source record or a verified per-dataset licence entry before ingestion.

## Potential use of `hyzd`
- Mandarin character readings;
- pronunciation mappings;
- Traditional/Simplified character support where present;
- cross-checking character readings/forms.

## Update method
Track exact repository/commit for each imported dataset and retain the specific repository’s licence/attribution metadata.

Last reviewed: 2026-10-06
