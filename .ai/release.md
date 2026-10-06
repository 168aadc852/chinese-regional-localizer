# Release / Packaging Tasks

Start with:
- the release artifact/configuration being changed;
- `docs/LICENSE_PACKAGING.md`;
- `data-registry/sources.yaml`.

Load on demand:
- individual source records only for data actually included in the release;
- build/update workflow files only when they are affected.

Rules:
- keep code and third-party data licences distinct;
- preserve attribution/ShareAlike pack boundaries;
- never include pending/reference-only/rejected data in redistributable builds;
- record exact source versions/checksums where applicable;
- verify tests and integrity checks before release.

Do not preload all source-review records.
