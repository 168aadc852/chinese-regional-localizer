#!/usr/bin/env python3
"""Import the LSHK Jyutping Table into the project SQLite schema.

The importer is offline-friendly and update-safe. Network access occurs only
when --download is explicitly requested. The production baseline is pinned to
a specific upstream commit and Git blob.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import sqlite3
import sys
import urllib.request
from pathlib import Path
from typing import Any, Iterable

import poc_builder

SOURCE_ID = "lshk-jyutping-table"
RESOURCE_ID = "lshk:list.tsv"
PINNED_COMMIT = "dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3"
PINNED_GIT_BLOB = "522f41701dd10c4da08d82923ef7ae40d14b9ffb"
PINNED_URL = (
    "https://raw.githubusercontent.com/lshk-org/jyutping-table/"
    f"{PINNED_COMMIT}/list.tsv"
)
EXPECTED_COLUMNS = ["CH", "UCODE", "JP", "INIT", "FINL", "TONE", "DESC", "DESC_JP"]
UCODE_RE = re.compile(r"^U\+[0-9A-Fa-f]{4,6}$")


class LshkFormatError(ValueError):
    """Raised when the TSV does not match the reviewed LSHK format."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_blob_sha1(path: Path) -> str:
    data = path.read_bytes()
    digest = hashlib.sha1()  # noqa: S324 - Git blob identity uses SHA-1 by definition.
    digest.update(f"blob {len(data)}\0".encode("ascii"))
    digest.update(data)
    return digest.hexdigest()


def verify_pinned_blob(path: Path) -> None:
    actual = git_blob_sha1(path)
    if actual != PINNED_GIT_BLOB:
        raise LshkFormatError(
            f"Pinned LSHK blob mismatch: expected {PINNED_GIT_BLOB}, got {actual}"
        )


def download_pinned(destination: Path) -> Path:
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        PINNED_URL,
        headers={"User-Agent": "chinese-regional-localizer/phase-2c"},
    )
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("wb") as output:
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
    verify_pinned_blob(destination)
    return destination


def iter_rows(path: Path) -> Iterable[dict[str, str]]:
    with path.open("r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames != EXPECTED_COLUMNS:
            raise LshkFormatError(
                f"Unexpected TSV header. Expected {EXPECTED_COLUMNS!r}, got {reader.fieldnames!r}"
            )
        for line_number, row in enumerate(reader, start=2):
            if None in row:
                raise LshkFormatError(f"Line {line_number}: unexpected extra columns")
            if any(row.get(column) is None for column in EXPECTED_COLUMNS):
                raise LshkFormatError(f"Line {line_number}: missing column value")
            character = row["CH"]
            ucode = row["UCODE"]
            reading = row["JP"]
            tone = row["TONE"]
            if len(character) != 1:
                raise LshkFormatError(f"Line {line_number}: CH must be exactly one Unicode character")
            if not UCODE_RE.match(ucode):
                raise LshkFormatError(f"Line {line_number}: invalid UCODE {ucode!r}")
            expected_cp = int(ucode[2:], 16)
            if ord(character) != expected_cp:
                raise LshkFormatError(
                    f"Line {line_number}: CH {character!r} does not match UCODE {ucode}"
                )
            if not reading:
                raise LshkFormatError(f"Line {line_number}: JP is empty")
            if tone and not tone.isdigit():
                raise LshkFormatError(f"Line {line_number}: invalid TONE {tone!r}")
            yield {key: (value or "") for key, value in row.items()}


def ensure_schema(conn: sqlite3.Connection, migration_path: Path) -> None:
    conn.execute("PRAGMA foreign_keys = ON")
    conn.executescript(migration_path.read_text(encoding="utf-8"))


def open_or_create_database(
    db_path: Path,
    schema_path: Path,
    migration_path: Path,
    manifest: dict[str, Any],
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
    poc_builder.ensure_hardening_schema(conn, schema_path)
    ensure_schema(conn, migration_path)
    poc_builder.load_manifest_sources(conn, manifest)
    return conn


def add_source_version(
    conn: sqlite3.Connection,
    *,
    revision_id: str,
    upstream_url: str,
    checksum_sha256: str,
    retrieved_at: str,
    note: str,
) -> tuple[int, bool]:
    existing = poc_builder.find_current_source_version(
        conn,
        source_id=SOURCE_ID,
        resource_key=RESOURCE_ID,
        revision_id=revision_id,
        checksum_sha256=checksum_sha256,
    )
    if existing is not None:
        return existing, False

    conn.execute(
        """
        UPDATE source_versions
        SET is_current = 0
        WHERE source_id = ? AND is_current = 1
          AND (resource_key = ? OR resource_key = '')
        """,
        (SOURCE_ID, RESOURCE_ID),
    )
    return poc_builder.register_source_version(
        conn,
        source_id=SOURCE_ID,
        resource_key=RESOURCE_ID,
        version_label=revision_id[:12],
        revision_id=revision_id,
        published_at=None,
        retrieved_at=retrieved_at,
        upstream_url=upstream_url,
        checksum_sha256=checksum_sha256,
        notes=note,
    )


def _ensure_character_evidence(
    conn: sqlite3.Connection,
    *,
    localized_name_id: int,
    ucode: str,
    source_version_id: int,
    upstream_url: str,
) -> None:
    existing = conn.execute(
        """
        SELECT 1 FROM name_evidence
        WHERE localized_name_id = ? AND source_id = ? AND source_version_id = ?
          AND upstream_record_id = ? AND evidence_type = 'source_character'
        """,
        (localized_name_id, SOURCE_ID, source_version_id, ucode),
    ).fetchone()
    if existing:
        return
    conn.execute(
        """
        INSERT INTO name_evidence (
            localized_name_id, source_id, source_version_id,
            upstream_record_id, upstream_url, evidence_type,
            confidence, transformation_note
        ) VALUES (?, ?, ?, ?, ?, 'source_character', 1.0, ?)
        """,
        (
            localized_name_id,
            SOURCE_ID,
            source_version_id,
            ucode,
            upstream_url,
            "Character identity recorded while importing LSHK pronunciation data.",
        ),
    )


def get_or_create_character(
    conn: sqlite3.Connection,
    character: str,
    ucode: str,
    source_version_id: int,
    upstream_url: str,
) -> int:
    row = conn.execute(
        "SELECT concept_id FROM external_ids WHERE namespace = 'unicode' AND external_value = ?",
        (ucode,),
    ).fetchone()
    if row:
        concept_id = int(row[0])
    else:
        cursor = conn.execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('character', ?, 'character_pronunciation')",
            (f"unicode:{ucode}",),
        )
        concept_id = int(cursor.lastrowid)
        conn.execute(
            "INSERT INTO external_ids (concept_id, namespace, external_value, source_id) VALUES (?, 'unicode', ?, ?)",
            (concept_id, ucode, SOURCE_ID),
        )

    name_row = conn.execute(
        """
        SELECT localized_name_id FROM localized_names
        WHERE concept_id = ? AND locale = 'zh-HK' AND text = ? AND name_type = 'character'
        """,
        (concept_id, character),
    ).fetchone()
    if name_row:
        localized_name_id = int(name_row[0])
    else:
        cursor = conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, 'zh-HK', ?, 'character', 'character_pronunciation', 1, 1.0)
            """,
            (concept_id, character),
        )
        localized_name_id = int(cursor.lastrowid)

    _ensure_character_evidence(
        conn,
        localized_name_id=localized_name_id,
        ucode=ucode,
        source_version_id=source_version_id,
        upstream_url=upstream_url,
    )
    return concept_id


def import_tsv(
    *,
    input_path: Path,
    db_path: Path,
    manifest_path: Path,
    schema_path: Path,
    migration_path: Path,
    revision_id: str,
    upstream_url: str,
    retrieved_at: str,
    reset: bool = False,
) -> dict[str, Any]:
    manifest = poc_builder.load_manifest(manifest_path)
    source = poc_builder.assert_source_resource_allowed(manifest, SOURCE_ID, RESOURCE_ID)
    if source["pack"] != "attribution":
        raise poc_builder.IngestPolicyError(
            f"Unexpected pack for {SOURCE_ID}: expected attribution, got {source['pack']}"
        )

    if revision_id == PINNED_COMMIT:
        verify_pinned_blob(input_path)

    rows = list(iter_rows(input_path))
    checksum = sha256_file(input_path)
    conn = open_or_create_database(db_path, schema_path, migration_path, manifest, reset)
    inserted_readings = 0
    characters: set[str] = set()
    try:
        poc_builder.assert_database_pack_compatible(conn, source["pack"])
        source_version_id, _ = add_source_version(
            conn,
            revision_id=revision_id,
            upstream_url=upstream_url,
            checksum_sha256=checksum,
            retrieved_at=retrieved_at,
            note=(
                f"LSHK Jyutping Table list.tsv; verified Git blob {PINNED_GIT_BLOB} "
                "when importing the pinned production revision. SHA-256 records exact bytes."
            ),
        )
        for row in rows:
            concept_id = get_or_create_character(
                conn,
                row["CH"],
                row["UCODE"],
                source_version_id,
                upstream_url,
            )
            cursor = conn.execute(
                """
                INSERT OR IGNORE INTO pronunciations (
                    concept_id, locale, romanization_scheme, reading,
                    initial, final, tone, description, description_romanized,
                    source_id, source_version_id, upstream_record_id,
                    upstream_url, confidence
                ) VALUES (?, 'yue-Hant-HK', 'Jyutping', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1.0)
                """,
                (
                    concept_id,
                    row["JP"],
                    row["INIT"] or None,
                    row["FINL"] or None,
                    row["TONE"] or None,
                    row["DESC"] or None,
                    row["DESC_JP"] or None,
                    SOURCE_ID,
                    source_version_id,
                    f"{row['UCODE']}:{row['JP']}",
                    upstream_url,
                ),
            )
            if cursor.rowcount:
                inserted_readings += 1
            characters.add(row["UCODE"])

        conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('lshk_last_revision', ?)",
            (revision_id,),
        )
        conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('lshk_last_sha256', ?)",
            (checksum,),
        )
        conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('schema_version', '0.2')"
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
        "revision_id": revision_id,
        "sha256": checksum,
        "characters": len(characters),
        "readings_inserted": inserted_readings,
        "db_path": str(db_path),
    }


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--input", type=Path, help="Import an already downloaded list.tsv")
    mode.add_argument("--download", action="store_true", help="Download the commit-pinned upstream list.tsv")
    parser.add_argument("--db", type=Path, default=repo_root / "build" / "lshk.sqlite")
    parser.add_argument("--reset", action="store_true", help="Recreate the database before importing")
    parser.add_argument("--revision", default=None, help="Revision identifier for local input")
    parser.add_argument("--retrieved-at", default=None)
    args = parser.parse_args()

    if args.download:
        input_path = (
            repo_root
            / "data"
            / "downloads"
            / "lshk-jyutping-table"
            / PINNED_COMMIT
            / "list.tsv"
        )
        download_pinned(input_path)
        revision_id = PINNED_COMMIT
        upstream_url = PINNED_URL
    else:
        input_path = args.input
        revision_id = args.revision or "local-input"
        upstream_url = f"file://{input_path.resolve()}"

    result = import_tsv(
        input_path=input_path,
        db_path=args.db,
        manifest_path=repo_root / "data-registry" / "sources.yaml",
        schema_path=repo_root / "schema" / "sqlite-v0.2.sql",
        migration_path=repo_root / "schema" / "migrations" / "0002_pronunciations.sql",
        revision_id=revision_id,
        upstream_url=upstream_url,
        retrieved_at=args.retrieved_at or poc_builder.utc_now_iso(),
        reset=args.reset,
    )
    print(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
