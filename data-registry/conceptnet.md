# ConceptNet

Status: **approved_with_conditions**

## Intended use
Chinese semantic relations, concept links, synonym/context support and ambiguity handling.

## Official source
Website: https://conceptnet.io/
Publisher: ConceptNet / Commonsense Computing Initiative

## Licence review
ConceptNet states that ConceptNet 5 data is licensed under **Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0)**.

The official site also provides a recommended attribution statement describing ConceptNet as compiled from multiple upstream sources including Wikimedia projects, WordNet and others.

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- ShareAlike: **Required** for adaptations
- Upstream provenance: Important because ConceptNet aggregates many sources

## Packaging decision
Do not merge ConceptNet-derived graph data into the permissive core database.

Use a separate ShareAlike semantic pack and preserve:
- ConceptNet URI/edge identifiers;
- relation type;
- language;
- source/provenance fields where present;
- CC BY-SA 4.0 attribution/licence notice.

For this project, ConceptNet should be treated as a semantic/context aid, not as the primary authority for regional Chinese naming.

## Update method
Use official ConceptNet data releases/downloads rather than scraping rendered pages. Record release version and source provenance.

Last reviewed: 2026-10-06
