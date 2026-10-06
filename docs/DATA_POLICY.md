# Data Policy

## Purpose

This policy controls what external data may be researched, imported, transformed, packaged or redistributed by the project.

## Fundamental rule

**Public access is not the same as open licensing.**

A source must not enter a redistributable project data pack merely because it can be viewed or queried online.

## Source statuses

### approved

Licence and intended use have been verified as compatible with the planned use.

### approved_with_conditions

Usable only under recorded conditions such as attribution, share-alike, separation into a distinct data pack or other obligations.

### reference_only

May be consulted for research or manual verification but must not be bulk imported or redistributed without additional permission.

### pending_review

Potentially useful, but licensing or technical conditions are not yet sufficiently verified.

### rejected

Known to be incompatible with the project use, or intentionally excluded.

## Required review fields

Before approval, record:

- official source name and publisher;
- official source URL;
- licence name and authoritative evidence URL;
- data/content covered by that licence;
- commercial-use permission;
- modification permission;
- redistribution permission;
- attribution requirements;
- share-alike/copyleft obligations;
- database-specific rights if relevant;
- API/download/scraping restrictions;
- update method and versioning;
- intended project use;
- review date and reviewer notes.

## Data-layer separation

The project should be designed so data with different licence obligations can remain separable.

Proposed conceptual layers:

- **Core/permissive layer** — data suitable for broad reuse with minimal restrictions.
- **Attribution layer** — data requiring source acknowledgement.
- **Share-alike layer** — derivative datasets that may carry share-alike obligations.
- **Reference-only layer** — never packaged into redistributable builds.
- **User layer** — local user-defined terminology and preferences.

## Provenance

Imported records should preserve provenance wherever practical, including source identifier, source URL or canonical ID, licence identifier, source revision/version and retrieval/update date.

## No licence laundering

Transforming, normalizing, converting formats or storing data in SQLite does not automatically remove original licence obligations.

## Scraping and APIs

Do not implement bulk scraping simply because a website is technically accessible. Review published terms, API policies, robots/access constraints and licence coverage first.

## Contributions

Community contributions should either:

- be original contributions under a clearly stated project-compatible data licence; or
- identify an approved source and preserve required provenance.

Unverifiable copied terminology should not be merged into the canonical database.

## Legal note

Repository documentation records project due diligence but is not a substitute for professional legal advice when required.
