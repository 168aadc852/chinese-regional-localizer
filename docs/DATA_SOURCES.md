# Data Sources Registry

This is the index. Detailed reviews live under `data-registry/`.

No source listed here should be assumed redistributable unless its status says `approved` or `approved_with_conditions` and the detailed record contains licensing evidence.

## Current candidates

| Source | Main use | Status | Record |
|---|---|---|---|
| OpenCC | Script and regional terminology conversion | approved_with_conditions | `data-registry/opencc.md` |
| Wikidata | Structured entities, labels and aliases | approved | `data-registry/wikidata.md` |
| Chinese Wikipedia | Regional names, conversion rules, article context | approved_with_conditions | `data-registry/wikipedia.md` |
| Taiwan Government / terminology open data | TW/CN professional terminology | approved_with_conditions | `data-registry/taiwan-government.md` |
| DATA.GOV.HK | Hong Kong official open datasets | approved_with_conditions | `data-registry/hk-government.md` |
| Combined DoJ Glossaries of Legal Terms | HK legal terminology | pending_review | `data-registry/doj-glossary.md` |
| CC-CEDICT | General lexical support | approved_with_conditions | `data-registry/cc-cedict.md` |
| Words.hk | HK/Cantonese lexical signals; public-domain subset only for core use | approved_with_conditions | `data-registry/words-hk.md` |
| Rime Cantonese | Cantonese/HK lexical and pronunciation data | approved_with_conditions | `data-registry/rime-cantonese.md` |
| Unicode / Unihan / CLDR | Character variants and locale data | approved_with_conditions | `data-registry/unicode-cldr.md` |
| MusicBrainz | Music entities and aliases; Core Data only | approved_with_conditions | `data-registry/musicbrainz.md` |
| THUOCL | Mainland/domain vocabulary and frequency signals | approved_with_conditions | `data-registry/thuocl.md` |
| PanLex | Multilingual lexical mappings and aliases | approved | `data-registry/panlex.md` |
| CC-Canto | Cantonese vocabulary, Jyutping and meanings | approved_with_conditions | `data-registry/cc-canto.md` |
| Chinese Wiktionary | Definitions, senses, usage labels and translations | approved_with_conditions | `data-registry/chinese-wiktionary.md` |
| DBnary | Structured Wiktionary lexical data / OntoLex RDF | pending_review | `data-registry/dbnary.md` |
| ConceptNet | Semantic relations and ambiguity/context support | approved_with_conditions | `data-registry/conceptnet.md` |
| Chinese Open WordNet | Chinese synsets and sense disambiguation | approved_with_conditions | `data-registry/chinese-open-wordnet.md` |
| HKCanCor | Hong Kong Cantonese words, Jyutping, POS and frequency evidence | approved_with_conditions | `data-registry/hkcancor.md` |
| LSHK Jyutping Table | Cantonese character readings and HKSCS support | approved_with_conditions | `data-registry/lshk-jyutping-table.md` |
| Kaifangcidian / Open Chinese Dictionary | Individually reviewed open dictionary/pronunciation datasets | approved_with_conditions | `data-registry/kaifangcidian.md` |
| CFDICT | Traditional/Simplified lexical pairs, Pinyin, Chinese-French meanings | approved_with_conditions | `data-registry/cfdict.md` |
| OpenHowNet | Sememes, word senses and semantic similarity | pending_review | `data-registry/openhownet.md` |

## Current packaging classes emerging from review

### Permissive/core candidates
- Wikidata structured data — CC0
- PanLex official snapshots — CC0
- OpenCC — Apache-2.0 with notice requirements
- Unicode/Unihan/CLDR machine-readable Data Files — Unicode-3.0
- Words.hk explicitly public-domain word-list/pronunciation dataset
- MusicBrainz Core Data — CC0 only
- THUOCL published repository word lists/frequency signals — MIT/open-use conditions recorded in its review
- Chinese Open WordNet — permissive custom licence with notice/disclaimer preservation

### Attribution-required / non-ShareAlike packs
- Verified Taiwan Government OGL-Taiwan v1 terminology datasets
- Qualifying DATA.GOV.HK data within portal terms
- Rime Cantonese CC BY 4.0 main content
- HKCanCor — CC BY 4.0
- LSHK Jyutping Table — CC BY 4.0
- Individually reviewed Kaifangcidian datasets such as `kfcd/hyzd` — CC BY 3.0

### ShareAlike-isolated packs
- Chinese Wikipedia — CC BY-SA 4.0-derived data
- Chinese Wiktionary — CC BY-SA 4.0 text-derived data
- CC-CEDICT — CC BY-SA 4.0
- CC-Canto — CC BY-SA 3.0
- ConceptNet — CC BY-SA 4.0
- CFDICT — CC BY-SA 3.0
- Rime `jyut6ping3.maps` — ODbL 1.0 database layer
- DBnary — clearly ShareAlike, but exact current licence version still requires snapshot-level verification

### Reference-only / excluded portions within otherwise useful sources
- Full Words.hk dictionary under Non-Commercial Open Data License — exclude from commercial-capable standard pack
- MusicBrainz Supplementary Data under CC BY-NC-SA 3.0 — exclude from standard pack

### Pending rights clarification
- Combined DoJ Glossaries — explicit reuse/adaptation rights not yet confirmed
- DBnary — current project surfaces show a CC BY-SA version inconsistency; verify exact snapshot licence before ingestion
- OpenHowNet downloadable HowNet core data — MIT clearly covers repository/API code, but data-specific redistribution rights are not yet independently confirmed

## Review status summary

- Total source records: **23**
- Formal first-pass reviews completed with usable conclusions: **20**
- Still pending rights/version clarification: **3**

## Review principle

Repository visibility or the word “open” in a project name is not sufficient evidence. Each source must be checked against authoritative licence text and the licence must be shown to cover the specific data files we intend to ingest.
