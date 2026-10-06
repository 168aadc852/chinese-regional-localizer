#!/usr/bin/env python3
"""Build the Phase 0.5 SQLite proof of concept.

This module is intentionally small and dependency-light. PyYAML is used only to
read the canonical source manifest. The PoC fixture records are hand-authored
and are not authoritative upstream extracts; see data/fixtures/README.md.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path
from typing import Any

import yaml


class IngestPolicyError(ValueError):
    """Raised when data attempts to enter the database from a disallowed source."""


REQUIRED_SOURCE_FIELDS = {
    "id",
    "name",
    "status",
    "pack",
    "licence",
    "commercial_use",
    "modification_allowed",
    "redistribution_allowed",
    "attribution_required",
    "share_alike",
    "ingest_allowed",
    "ingest_scope",
    "excluded_scope",
    "review_record",
}


def load_manifest(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        manifest = yaml.safe_load(handle)
    if not isinstance(manifest, dict):
        raise ValueError("Manifest root must be a mapping")
    validate_manifest(manifest)
    return manifest


def validate_manifest(manifest: dict[str, Any]) -> None:
    if not isinstance(manifest.get("manifest_version"), int):
        raise ValueError("manifest_version must be an integer")

    status_values = set(manifest.get("status_values", []))
    pack_values = set(manifest.get("pack_values", []))
    sources = manifest.get("sources")
    if not isinstance(sources, list) or not sources:
        raise ValueError("sources must be a non-empty list")

    seen: set[str] = set()
    for source in sources:
        if not isinstance(source, dict):
            raise ValueError("Every source must be a mapping")
        missing = REQUIRED_SOURCE_FIELDS - set(source)
        if missing:
            raise ValueError(f"Source is missing fields: {sorted(missing)}")
        source_id = source["id"]
        if source_id in seen:
            raise ValueError(f"Duplicate source id: {source_id}")
        seen.add(source_id)
        if source["status"] not in status_values:
            raise ValueError(f"Invalid status for {source_id}: {source['status']}")
        if source["pack"] not in pack_values:
            raise ValueError(f"Invalid pack for {source_id}: {source['pack']}")
        if not isinstance(source["ingest_allowed"], bool):
            raise ValueError(f"ingest_allowed must be boolean for {source_id}")
        if source["status"] in {"pending_review", "reference_only", "rejected"} and source["ingest_allowed"]:
            raise ValueError(f"Unsafe manifest: {source_id} cannot be ingest_allowed while {source['status']}")


def source_map(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {source["id"]: source for source in manifest["sources"]}


def assert_source_ingest_allowed(manifest: dict[str, Any], source_id: str) -> dict[str, Any]:
    sources = source_map(manifest)
    if source_id not in sources:
        raise IngestPolicyError(f"Unknown source: {source_id}")
    source = sources[source_id]
    if not source["ingest_allowed"]:
        raise IngestPolicyError(
            f"Source {source_id} is blocked by manifest policy "
            f"(status={source['status']}, pack={source['pack']})"
        )
    return source


def create_database(schema_path: Path, db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if db_path.exists():
        db_path.unlink()
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.executescript(schema_path.read_text(encoding="utf-8"))
    return conn


def bool_to_int(value: Any) -> int | None:
    if value is None:
        return None
    return 1 if value else 0


def load_manifest_sources(conn: sqlite3.Connection, manifest: dict[str, Any]) -> None:
    version = manifest["manifest_version"]
    updated = manifest.get("last_updated")
    for source in manifest["sources"]:
        conn.execute(
            """
            INSERT INTO sources (
                source_id, name, status, pack, licence,
                commercial_use, modification_allowed, redistribution_allowed,
                attribution_required, share_alike, ingest_allowed,
                ingest_scope, excluded_scope, review_record,
                manifest_version, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                source["id"],
                source["name"],
                source["status"],
                source["pack"],
                source["licence"],
                bool_to_int(source["commercial_use"]),
                bool_to_int(source["modification_allowed"]),
                bool_to_int(source["redistribution_allowed"]),
                bool_to_int(source["attribution_required"]),
                bool_to_int(source["share_alike"]),
                bool_to_int(source["ingest_allowed"]),
                source["ingest_scope"],
                source["excluded_scope"],
                source["review_record"],
                version,
                updated,
            ),
        )


def load_fixture(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError("Fixture root must be an object")
    return data


def insert_source_versions(
    conn: sqlite3.Connection,
    manifest: dict[str, Any],
    fixture: dict[str, Any],
) -> dict[str, int]:
    result: dict[str, int] = {}
    for item in fixture.get("source_versions", []):
        source_id = item["source_id"]
        assert_source_ingest_allowed(manifest, source_id)
        cursor = conn.execute(
            """
            INSERT INTO source_versions (
                source_id, version_label, revision_id, published_at,
                retrieved_at, upstream_url, checksum_sha256, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                source_id,
                item.get("version_label"),
                item.get("revision_id"),
                item.get("published_at"),
                item["retrieved_at"],
                item.get("upstream_url"),
                item.get("checksum_sha256"),
                "Synthetic PoC fixture; not an authoritative upstream extract.",
            ),
        )
        result[source_id] = int(cursor.lastrowid)
    return result


def insert_term_rules(
    conn: sqlite3.Connection,
    manifest: dict[str, Any],
    fixture: dict[str, Any],
    versions: dict[str, int],
) -> None:
    for rule in fixture.get("term_rules", []):
        source_id = rule["source_id"]
        assert_source_ingest_allowed(manifest, source_id)
        conn.execute(
            """
            INSERT INTO term_rules (
                source_locale, target_locale, source_text, target_text,
                domain, rule_type, priority, context_constraint,
                source_id, source_version_id, upstream_record_id,
                upstream_url, confidence, active
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            """,
            (
                rule.get("source_locale"),
                rule["target_locale"],
                rule["source_text"],
                rule["target_text"],
                rule.get("domain"),
                rule.get("rule_type", "poc_fixture"),
                rule.get("priority", 0),
                rule.get("context_constraint"),
                source_id,
                versions.get(source_id),
                "fixture:term-rule",
                f"fixture://{source_id}",
                rule.get("confidence"),
            ),
        )


def insert_concepts(
    conn: sqlite3.Connection,
    manifest: dict[str, Any],
    fixture: dict[str, Any],
    versions: dict[str, int],
) -> None:
    for concept in fixture.get("concepts", []):
        cursor = conn.execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain, created_at) VALUES (?, ?, ?, ?)",
            (
                concept["concept_type"],
                concept.get("canonical_key"),
                concept.get("domain"),
                "2026-10-06T00:00:00Z",
            ),
        )
        concept_id = int(cursor.lastrowid)

        for ext in concept.get("external_ids", []):
            source_id = ext.get("source_id")
            if source_id:
                assert_source_ingest_allowed(manifest, source_id)
            conn.execute(
                "INSERT INTO external_ids (concept_id, namespace, external_value, source_id) VALUES (?, ?, ?, ?)",
                (concept_id, ext["namespace"], ext["value"], source_id),
            )

        for name in concept.get("names", []):
            source_id = name["source_id"]
            assert_source_ingest_allowed(manifest, source_id)
            name_cursor = conn.execute(
                """
                INSERT INTO localized_names (
                    concept_id, locale, text, name_type, domain,
                    is_preferred, confidence
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    concept_id,
                    name["locale"],
                    name["text"],
                    name["name_type"],
                    concept.get("domain"),
                    1 if name.get("is_preferred") else 0,
                    name.get("confidence", 1.0),
                ),
            )
            localized_name_id = int(name_cursor.lastrowid)
            conn.execute(
                """
                INSERT INTO name_evidence (
                    localized_name_id, source_id, source_version_id,
                    upstream_record_id, upstream_url, upstream_revision,
                    evidence_type, confidence, transformation_note, retrieved_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    localized_name_id,
                    source_id,
                    versions.get(source_id),
                    f"fixture:{concept.get('canonical_key', concept_id)}",
                    f"fixture://{source_id}",
                    None,
                    "poc_fixture",
                    name.get("confidence", 1.0),
                    "Hand-authored PoC fixture; not authoritative upstream evidence.",
                    "2026-10-06T00:00:00Z",
                ),
            )


def write_build_metadata(conn: sqlite3.Connection, manifest: dict[str, Any]) -> None:
    values = {
        "build_type": "phase-0.5-poc",
        "manifest_version": str(manifest["manifest_version"]),
        "manifest_last_updated": str(manifest.get("last_updated", "")),
        "fixture_data_authoritative": "false",
    }
    conn.executemany(
        "INSERT OR REPLACE INTO build_metadata (key, value) VALUES (?, ?)",
        values.items(),
    )


def build_database(
    manifest_path: Path,
    schema_path: Path,
    fixture_path: Path,
    db_path: Path,
) -> Path:
    manifest = load_manifest(manifest_path)
    fixture = load_fixture(fixture_path)
    conn = create_database(schema_path, db_path)
    try:
        load_manifest_sources(conn, manifest)
        versions = insert_source_versions(conn, manifest, fixture)
        insert_term_rules(conn, manifest, fixture, versions)
        insert_concepts(conn, manifest, fixture, versions)
        write_build_metadata(conn, manifest)
        conn.commit()
        integrity = conn.execute("PRAGMA integrity_check").fetchone()[0]
        if integrity != "ok":
            raise RuntimeError(f"SQLite integrity_check failed: {integrity}")
    finally:
        conn.close()
    return db_path


def demo_summary(db_path: Path) -> dict[str, Any]:
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    try:
        software = [
            dict(row)
            for row in conn.execute(
                """
                SELECT target_locale, target_text, source_id
                FROM term_rules
                WHERE source_text = '软件'
                ORDER BY target_locale
                """
            )
        ]
        entities = [
            dict(row)
            for row in conn.execute(
                """
                SELECT c.canonical_key, c.concept_type, n.locale, n.text,
                       e.source_id, e.evidence_type, e.upstream_url
                FROM concepts c
                JOIN localized_names n ON n.concept_id = c.concept_id
                JOIN name_evidence e ON e.localized_name_id = n.localized_name_id
                WHERE c.canonical_key IN ('poc:inside-out', 'poc:brad-pitt', 'poc:hk-taxi')
                ORDER BY c.canonical_key, n.locale
                """
            )
        ]
        return {"software_rules": software, "entity_names": entities}
    finally:
        conn.close()


def default_paths(repo_root: Path) -> tuple[Path, Path, Path, Path]:
    return (
        repo_root / "data-registry" / "sources.yaml",
        repo_root / "schema" / "sqlite-v0.1.sql",
        repo_root / "data" / "fixtures" / "poc_records.json",
        repo_root / "build" / "localizer-poc.sqlite",
    )


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    manifest_default, schema_default, fixture_default, db_default = default_paths(repo_root)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=manifest_default)
    parser.add_argument("--schema", type=Path, default=schema_default)
    parser.add_argument("--fixtures", type=Path, default=fixture_default)
    parser.add_argument("--db", type=Path, default=db_default)
    parser.add_argument("--demo", action="store_true", help="Print example lookups after building")
    args = parser.parse_args()

    output = build_database(args.manifest, args.schema, args.fixtures, args.db)
    print(f"Built PoC database: {output}")
    if args.demo:
        print(json.dumps(demo_summary(output), ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
