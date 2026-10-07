# Data Source Review Progress

Last updated: 2026-10-07

Machine manifest source IDs: **24**.  
Source IDs with usable first-pass conclusions: **24**.  
Still pending: **0**.

## Approved

- Wikidata structured data — CC0.
- PanLex official snapshots — CC0.

## Approved with conditions

OpenCC; Chinese Wikipedia; verified Taiwan Government/NAER datasets; DATA.GOV.HK portal data; CC-CEDICT; Words.hk public-domain subset; Rime Cantonese main and map layers as separate manifest IDs; Unicode/Unihan/CLDR data files; MusicBrainz Core Data; THUOCL; CC-Canto; Chinese Wiktionary; ConceptNet; Chinese Open WordNet; HKCanCor; LSHK Jyutping Table; reviewed Kaifangcidian `kfcd/hyzd`; CFDICT; and the official OpenHowNet-provided HowNet core-data download under the project's MIT licence statement.

DATA.GOV.HK remains a valid conditional review conclusion, but automated derived-database ingestion is disabled because general modification/adaptation permission remains unresolved. A specific dataset can be enabled later only after narrower review and an exact machine resource ID.

OpenHowNet core data is approved only for the official OpenHowNet-provided download covered by the official MIT licence statement. Automated ingestion remains disabled until the exact downloadable resource is pinned and importer validation is defined.

## Reference only

- Combined DoJ Glossaries of Legal Terms — public XML access is verified, but the reuse/adaptation and redistribution rights needed for a derived terminology database were not established.
- DBnary — ShareAlike reuse is clear in principle, but the exact licence version for a specific Chinese snapshot was not pinned; packaging remains blocked until snapshot-specific verification.

Reference-only records remain useful for research/manual verification but have `ingest_allowed: false` and are excluded from redistributable builds.

`data-registry/sources.yaml` is the machine policy; individual `data-registry/*.md` files contain the detailed evidence.
