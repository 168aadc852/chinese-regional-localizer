# Proof-of-concept fixtures

The files in this directory are **hand-authored test fixtures** used only to validate database structure, manifest enforcement, regional-name lookup and provenance plumbing.

They are **not authoritative extracts** from the named upstream datasets and must not be shipped as production terminology data.

Where a fixture references an approved `source_id`, it means "exercise the code path for data intended to come from this source class". Every inserted fixture evidence row is explicitly marked `poc_fixture` and uses a `fixture://` upstream URL.

Production importers must replace these fixtures with actual downloaded records and preserve real upstream identifiers, versions, URLs, checksums and retrieval timestamps.
