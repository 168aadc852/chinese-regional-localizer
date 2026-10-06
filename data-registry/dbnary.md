# DBnary

Status: **pending_review**

## Intended use
Structured Chinese lexical senses, translations, lexical relations and morphology in RDF/OntoLex format, as an alternative to parsing Wiktionary wikitext directly.

## Official sources
Project: https://kaiko.getalp.org/about-dbnary/
Downloads: https://kaiko.getalp.org/about-dbnary/download/

## Licence review
DBnary is derived from Wiktionary and clearly intends redistribution under a Creative Commons Attribution-ShareAlike licence.

However, authoritative/current project surfaces reviewed in 2026 show a **licence-version inconsistency**:
- the project home page states DBnary is distributed under **CC BY-SA 3.0**;
- current RDF/ontology metadata surfaces may identify **CC BY-SA 4.0**.

Because licence version affects compliance/compatibility, do not collapse this ambiguity into a guessed result.

Known implications regardless of 3.0 vs 4.0:
- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- ShareAlike: **Required**

## Packaging decision
Keep `pending_review` until the exact licence attached to the specific Chinese DBnary snapshot/files to be imported is verified.

When resolved, DBnary should live in a **ShareAlike-isolated lexical pack**, not the permissive core.

## Update method
Use official language-specific Turtle downloads. Record:
- exact file URL;
- download date/snapshot date;
- licence URI embedded in or published for that snapshot;
- DBnary version;
- source Wiktionary edition.

Last reviewed: 2026-10-06
