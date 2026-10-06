#!/usr/bin/env python3
"""Import selected Wikidata EntityData JSON into the local SQLite schema.

Normal tests/builds can use local JSON. Network access occurs only when
--download-qid is explicitly supplied. The importer keeps only a small locale
allowlist and never fabricates missing regional labels.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import sys
import urllib.request
from pathlib import Path
from typing import Any, Iterable

import poc_builder

SOURCE_ID = "wikidata"
QID_RE = re.compile(r"^Q[1-9][0-9]*$")
ENTITY_URL = "https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"

LOCALE_MAP = {
    "en": "en",
    "mul": "mul",
    "zh": "zh",
    "zh-hans": "zh-Hans",
    "zh-hant": "zh-Hant",
    "zh-cn": "zh-CN",
    "zh-hk": "zh-HK",
    "zh-tw": "zh-TW",
}


class WikidataFormatError(ValueError):
    """Raised when EntityData JSON is malformed or inconsistent."""


def canonical_sha256(value: Any) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def download_latest(qid: str, destination: Path) -> Path:
    if not QID_RE.match(qid):
        raise ValueError(f"Invalid Wikidata QID: {qid}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        ENTITY_URL.format(qid=qid),
        headers={"User-Agent": "chinese-regional-localizer/phase-1c"},
    )
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("wb") as output:
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
    return destination


def iter_entities(document: Any) -> Iterable[tuple[str, dict[str, Any]]]:
    if not isinstance(document, dict) or not isinstance(document.get("entities"), dict):
        raise WikidataFormatError("Top-level JSON must contain an 'entities' object")

    for key, entity in document["entities"].items():
        if not isinstance(key, str) or not QID_RE.match(key):
            raise WikidataFormatError(f"Invalid entity key: {key!r}")
        if not isinstance(entity, dict):
            raise WikidataFormatError(f"{key}: entity must be an object")
        if entity.get("id") != key:
            raise WikidataFormatError(f"{key}: entity id does not match key")
        if entity.get("type") != "item":
            raise WikidataFormatError(f"{key}: only Wikidata item entities are supported")
        lastrevid = entity.get("lastrevid")
        if not isinstance(lastrevid, int) or lastrevid <= 0:
            raise WikidataFormatError(f"{key}: missing/invalid positive integer lastrevid")
        for field in ("labels", "aliases"):
            value = entity.get(field, {})
            if not isinstance(value, dict):
                raise WikidataFormatError(f"{key}: {field} must be an object")
        yield key, entity


def normalize_locale(code: str) -> str | None:
    return LOCALE_MAP.get(code.lower())


def iter_names(entity: dict[str, Any]) -> Iterable[tuple[str, str, str, bool, float]]:
    labels = entity.get("labels", {})
    for source_locale, payload in labels.items():
        locale = normalize_locale(source_locale)
        if locale is None:
            continue
        if not isinstance(payload, dict):
            raise WikidataFormatError(f"{entity['id']}: label {source_locale} must be an object")
        if payload.get("language", "").lower() != source_locale.lower():
            raise WikidataFormatError(f"{entity['id']}: label language mismatch for {source_locale}")
        text = payload.get("value")
        if not isinstance(text, str) or not text:
            raise WikidataFormatError(f"{entity['id']}: empty/invalid label for {source_locale}")
        yield locale, text, "preferred", True, 1.0

    aliases = entity.get("aliases", {})
    for source_locale, items in aliases.items():
        locale = normalize_locale(source_locale)
        if locale is None:
            continue
        if not isinstance(items, list):
            raise WikidataFormatError(f"{entity['id']}: aliases {source_locale} must be a list")
        for item in items:
            if not isinstance(item, dict):
                raise WikidataFormatError(f"{entity['id']}: alias item must be an object")
            if item.get("language", "").lower() != source_locale.lower():
                raise WikidataFormatError(f"{entity['id']}: alias language mismatch for {source_locale}")
            text = item.get("value")
            if not isinstance(text, str) or not text:
                raise WikidataFormatError(f"{entity['id']}: empty/invalid alias for {source_locale}")
            yield locale, text, "alias", False, 0.95


def open_or_create_database(
    db_path: Path,
    schema_path: Path,
    manifest: dict[str, Any],
    *,
    reset: bool,
) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if reset and db_path.exists():
        db_path.unlink()
    new_db = not db_path.exists()
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    if new_db:
        conn.executescript(schema_path.read_text(encoding="utf-8"))
        poc_builder.load_manifest_sources(conn, manifest)
    return conn


def get_or_create_concept(
    conn: sqlite3.Connection,
    *,
    qid: str,
    concept_type: str,
) -> int:
    row = conn.execute(
        """
        SELECT c.concept_id, c.concept_type
        FROM external_ids e
        JOIN concepts c ON c.concept_id = e.concept_id
        WHERE e.namespace = 'wikidata' AND e.external_value = ?
        """,
        (qid,),
    ).fetchone()
    if row:
        if row["concept_type"] != concept_type:
            raise ValueError(
                f"{qid}: existing concept type {row['concept_type']!r} conflicts with {concept_type!r}"
            )
        return int(row["concept_id"])

    cursor = conn.execute(
        "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES (?, ?, 'entity')",
        (concept_type, f"wikidata:{qid}"),
    )
    concept_id = int(cursor.lastrowid)
    conn.execute(
        """
        INSERT INTO external_ids (concept_id, namespace, external_value, source_id)
        VALUES (?, 'wikidata', ?, ?)
        """,
        (concept_id, qid, SOURCE_ID),
    )
    return concept_id


def add_source_version(
    conn: sqlite3.Connection,
    *,
    qid: str,
    entity: dict[str, Any],
    checksum: str,
    retrieved_at: str,
    fixture_mode: bool,
) -> tuple[int, str]:
    revision = str(entity["lastrevid"])
    if fixture_mode:
        upstream_url = f"fixture://wikidata/{qid}.json"
    else:
        upstream_url = f"{ENTITY_URL.format(qid=qid)}?revision={revision}"
    cursor = conn.execute(
        """
        INSERT INTO source_versions (
            source_id, version_label, revision_id, published_at, retrieved_at,
            upstream_url, checksum_sha256, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            SOURCE_ID,
            f"{qid}:rev-{revision}",
            revision,
            entity.get("modified"),
            retrieved_at,
            upstream_url,
            checksum,
            f"Wikidata structured entity JSON for {qid}; CC0 scope.",
        ),
    )
    return int(cursor.lastrowid), upstream_url


def add_name_and_evidence(
    conn: sqlite3.Connection,
    *,
    concept_id: int,
    qid: str,
    locale: str,
    text: str,
    name_type: str,
    is_preferred: bool,
    confidence: float,
    source_version_id: int,
    upstream_url: str,
    revision: str,
) -> None:
    row = conn.execute(
        """
        SELECT localized_name_id FROM localized_names
        WHERE concept_id = ? AND locale = ? AND text = ? AND name_type = ?
        """,
        (concept_id, locale, text, name_type),
    ).fetchone()
    if row:
        localized_name_id = int(row[0])
    else:
        cursor = conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, ?, ?, ?, 'entity', ?, ?)
            """,
            (concept_id, locale, text, name_type, int(is_preferred), confidence),
        )
        localized_name_id = int(cursor.lastrowid)

    existing = conn.execute(
        """
        SELECT 1 FROM name_evidence
        WHERE localized_name_id = ? AND source_id = ? AND source_version_id = ?
          AND upstream_record_id = ? AND evidence_type = ?
        """,
        (
            localized_name_id,
            SOURCE_ID,
            source_version_id,
            f"{qid}:{locale}:{name_type}:{text}",
            "direct_label" if name_type == "preferred" else "alias",
        ),
    ).fetchone()
    if not existing:
        conn.execute(
            """
            INSERT INTO name_evidence (
                localized_name_id, source_id, source_version_id,
                upstream_record_id, upstream_url, upstream_revision,
                evidence_type, confidence, transformation_note, retrieved_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
            """,
            (
                localized_name_id,
                SOURCE_ID,
                source_version_id,
                f"{qid}:{locale}:{name_type}:{text}",
                upstream_url,
                revision,
                "direct_label" if name_type == "preferred" else "alias",
                confidence,
                "Locale code normalized only; no name translation or fallback was synthesized.",
            ),
        )


def import_document(
    *,
    input_path: Path,
    type_map_path: Path,
    db_path: Path,
    manifest_path: Path,
    schema_path: Path,
    retrieved_at: str,
    reset: bool = False,
    fixture_mode: bool = False,
) -> dict[str, Any]:
    manifest = poc_builder.load_manifest(manifest_path)
    source = poc_builder.assert_source_ingest_allowed(manifest, SOURCE_ID)
    if source["pack"] != "core":
        raise poc_builder.IngestPolicyError(
            f"Unexpected pack for {SOURCE_ID}: expected core, got {source['pack']}"
        )

    document = load_json(input_path)
    type_map = load_json(type_map_path)
    if not isinstance(type_map, dict):
        raise ValueError("Entity type map must be a JSON object keyed by QID")

    parsed_entities = list(iter_entities(document))
    for qid, _ in parsed_entities:
        if qid not in type_map or not isinstance(type_map[qid], str) or not type_map[qid]:
            raise ValueError(f"Missing explicit concept type for {qid}")

    conn = open_or_create_database(db_path, schema_path, manifest, reset=reset)
    imported_entities = 0
    imported_names = 0
    try:
        for qid, entity in parsed_entities:
            checksum = canonical_sha256(entity)
            source_version_id, upstream_url = add_source_version(
                conn,
                qid=qid,
                entity=entity,
                checksum=checksum,
                retrieved_at=retrieved_at,
                fixture_mode=fixture_mode,
            )
            concept_id = get_or_create_concept(
                conn,
                qid=qid,
                concept_type=type_map[qid],
            )
            for locale, text, name_type, is_preferred, confidence in iter_names(entity):
                add_name_and_evidence(
                    conn,
                    concept_id=concept_id,
                    qid=qid,
                    locale=locale,
                    text=text,
                    name_type=name_type,
                    is_preferred=is_preferred,
                    confidence=confidence,
                    source_version_id=source_version_id,
                    upstream_url=upstream_url,
                    revision=str(entity["lastrevid"]),
                )
                imported_names += 1
            imported_entities += 1

        conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('wikidata_entity_import_count', ?)",
            (str(imported_entities),),
        )
        conn.commit()
        integrity = conn.execute("PRAGMA integrity_check").fetchone()[0]
        if integrity != "ok":
            raise RuntimeError(f"SQLite integrity_check failed: {integrity}")
    except Exception:
        conn.rollback()
        raise
    finally:
        conn.close()

    return {
        "source_id": SOURCE_ID,
        "entities": imported_entities,
        "names": imported_names,
        "db_path": str(db_path),
    }


def combine_downloaded_documents(paths: list[Path]) -> dict[str, Any]:
    combined: dict[str, Any] = {"entities": {}}
    for path in paths:
        document = load_json(path)
        for qid, entity in iter_entities(document):
            if qid in combined["entities"]:
                raise WikidataFormatError(f"Duplicate entity across downloads: {qid}")
            combined["entities"][qid] = entity
    return combined


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--input", type=Path, help="Local Wikidata EntityData JSON document")
    mode.add_argument(
        "--download-qid",
        action="append",
        default=None,
        help="Explicitly download latest EntityData JSON for a QID; repeat for multiple QIDs",
    )
    parser.add_argument("--type-map", type=Path, required=True)
    parser.add_argument("--db", type=Path, default=repo_root / "build" / "wikidata.sqlite")
    parser.add_argument("--reset", action="store_true")
    parser.add_argument("--retrieved-at", default="2026-10-06T00:00:00Z")
    parser.add_argument("--fixture-mode", action="store_true", help="Mark provenance URLs as fixture:// (tests only)")
    args = parser.parse_args()

    if args.download_qid:
        download_dir = repo_root / "data" / "downloads" / "wikidata"
        paths = []
        for qid in args.download_qid:
            destination = download_dir / f"{qid}.json"
            download_latest(qid, destination)
            paths.append(destination)
        combined = combine_downloaded_documents(paths)
        input_path = download_dir / "selected-entities.json"
        input_path.write_text(json.dumps(combined, ensure_ascii=False, indent=2), encoding="utf-8")
        fixture_mode = False
    else:
        input_path = args.input
        fixture_mode = args.fixture_mode

    result = import_document(
        input_path=input_path,
        type_map_path=args.type_map,
        db_path=args.db,
        manifest_path=repo_root / "data-registry" / "sources.yaml",
        schema_path=repo_root / "schema" / "sqlite-v0.1.sql",
        retrieved_at=args.retrieved_at,
        reset=args.reset,
        fixture_mode=fixture_mode,
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
