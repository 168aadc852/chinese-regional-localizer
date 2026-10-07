# Data Sources Registry

Detailed reviews live under `data-registry/`; `data-registry/sources.yaml` is the machine-readable ingestion policy.

The manifest currently contains **24 source IDs**. Rime Cantonese is deliberately split into main CC-BY content and ODbL map content, so the manifest has one more machine source ID than the human-facing source-family count suggests.

## Core/permissive candidates

- OpenCC — Apache-2.0, exact reviewed dictionary resources only.
- Wikidata structured item data — CC0.
- PanLex official snapshots — CC0.
- Unicode/Unihan/CLDR machine-readable data files — Unicode-3.0.
- Words.hk explicitly public-domain subset only.
- MusicBrainz Core Data — CC0 only.
- THUOCL reviewed repository lists/signals.
- Chinese Open WordNet under its recorded permissive notice.
- OpenHowNet official core-data download — MIT, preserving the MIT notice and exact resource provenance; automated ingestion remains disabled until the exact download resource is pinned.

## Attribution packs

- Verified Taiwan Government OGL-Taiwan v1 datasets.
- Rime Cantonese main CC-BY content.
- HKCanCor.
- LSHK Jyutping Table.
- Reviewed Kaifangcidian `kfcd/hyzd`.

DATA.GOV.HK has a conditional human review conclusion, but automated derived-database ingestion is **disabled** until an individual dataset has clear modification/adaptation rights and an exact machine-readable resource approval.

## Share-alike isolated packs

Chinese Wikipedia, Chinese Wiktionary, CC-CEDICT, CC-Canto, ConceptNet, CFDICT and Rime `jyut6ping3.maps` remain separated from permissive/core releases. DBnary is also ShareAlike in principle but remains pending because the exact licence version/snapshot is unresolved.

## Excluded portions

- full Words.hk dictionary under its non-commercial licence;
- MusicBrainz Supplementary Data under CC BY-NC-SA;
- unrelated/historical/proprietary HowNet distributions not verified as covered by the OpenHowNet MIT statement;
- any source/file outside the exact reviewed scope;
- any production resource not listed in `ingest_resources` for an implemented importer.

## Pending

- Combined DoJ Glossaries;
- DBnary snapshot licence.

## Review principle

Repository visibility, public accessibility or the word “open” is not permission. Approval must cover the specific data resource and intended transformation/redistribution. Production importers require both source approval and an exact machine resource allow-list entry.
