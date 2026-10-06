# Test fixtures

Fixtures in this directory exist only for automated tests and development builds. They must not be shipped as authoritative terminology/data releases.

Fixture provenance must be classified per file/subdirectory:

- **synthetic** — hand-authored solely to exercise a format/behavior;
- **derived-format sample** — small values/records based on an approved upstream format or reviewed open-source data and therefore retaining that source/licence context.

Current classification:

- `poc_records.json`, `evaluation_cases.json`, `wikidata/` — synthetic/non-authoritative behavior fixtures;
- `lshk_sample.tsv` — synthetic rows in the reviewed LSHK TSV format;
- `opencc/*.txt` — small OpenCC-format/reference rows carrying the upstream Apache-2.0 header and used only for regression behavior.

Fixture URLs use `fixture://` where evidence rows are created. Production importers replace fixtures with validated upstream resources and record real revision/resource/checksum/retrieval provenance.
