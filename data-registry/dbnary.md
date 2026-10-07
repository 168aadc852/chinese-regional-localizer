# DBnary

Status: **reference_only**

## Intended use
Structured Chinese lexical senses, translations, lexical relations and morphology in RDF/OntoLex format, as an alternative to parsing Wiktionary wikitext directly.

## Official sources
Project: https://kaiko.getalp.org/about-dbnary/
Downloads: https://kaiko.getalp.org/about-dbnary/download/

## Licence review
DBnary is derived from Wiktionary and clearly intends redistribution under a Creative Commons Attribution-ShareAlike licence.

However, authoritative/current project surfaces reviewed in 2026 show a **licence-version inconsistency**:
- the project home page states DBnary is distributed under **CC BY-SA 3.0**;
- current RDF/ontology metadata surfaces may identify **CC BY-SA 4.0**;
- the current Chinese downloadable snapshot can be identified, but this review did not establish an authoritative exact licence URI/version tied unambiguously to that specific snapshot.

Known implications regardless of 3.0 vs 4.0:
- Commercial use: **Yes in principle**
- Modification: **Yes in principle**
- Redistribution: **Yes in principle**
- Attribution: **Required**
- ShareAlike: **Required**

The unresolved version still matters for exact compliance, notices and compatibility, so the project does not guess the applicable version.

## Packaging decision
**Reference only for now. Do not ingest a DBnary snapshot into a redistributable project pack until the exact snapshot licence/version is pinned from authoritative evidence.**

If a future exact Chinese snapshot is verified, it should live in a **ShareAlike-isolated lexical pack**, not the permissive core.

## Update method
Use official language-specific Turtle downloads only after snapshot-specific licence verification. Record:
- exact file URL;
- download date/snapshot date;
- authoritative licence URI/version for that snapshot;
- DBnary version;
- source Wiktionary edition;
- checksum/provenance.

Last reviewed: 2026-10-07
