# CC-CEDICT

Status: **approved_with_conditions**

## Intended use
General Chinese lexical support, traditional/simplified pairs, Pinyin and English definitions.

## Region coverage
General Chinese

## Official source
URL: https://cc-cedict.org/editor/editor.php?handler=Download
Publisher: CC-CEDICT community project

## Licence review
Licence: **Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0)**
Evidence: official CC-CEDICT download page
Licence text: https://creativecommons.org/licenses/by-sa/4.0/

The official download page explicitly states that the work is licensed under CC BY-SA 4.0 and may be used for non-commercial and commercial purposes provided attribution is given and improvements/additions are shared under the same licence.

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Indicate changes: **Required for adapted material**
- Share-alike/copyleft: **Yes** for adaptations
- Database-specific obligations: Treat any adapted/combined CC-CEDICT-derived lexical dataset as ShareAlike material unless legal analysis establishes a different boundary
- Download restrictions: Prefer official release downloads; distinguish verified releases from latest non-verified editing snapshots

## Packaging decision
Do not merge CC-CEDICT-derived content into the permissive core database.

Use a separate ShareAlike-compatible lexical data pack, with:
- source attribution;
- licence link/text;
- source version/date;
- indication of modifications;
- clear separation from CC0 / Apache / permissive data.

## Update method
Download official release snapshots from the recommended release channel referenced by CC-CEDICT. Record release date/hash where practical.

Last reviewed: 2026-10-06
