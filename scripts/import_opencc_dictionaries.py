#!/usr/bin/env python3
"""Import reviewed OpenCC dictionaries into the project SQLite schema.

The importer preserves staged conversion, dictionary precedence, candidate
ordering, provenance and update history. Re-importing the same snapshot is
idempotent; a new snapshot supersedes the previous active rules for that
resource without deleting history.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import sys
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

import poc_builder

SOURCE_ID = "opencc"
PINNED_COMMIT = "3ac34aa439a9908dd49fa92b5174b46314787ac2"
BASE_RAW_URL = f"https://raw.githubusercontent.com/BYVoid/OpenCC/{PINNED_COMMIT}/data/dictionary"


@dataclass(frozen=True)
class DictionarySpec:
    filename: str
    source_locale: str
    target_locale: str
    rule_type: str
    stage: str
    base_priority: int

    @property
    def pinned_url(self) -> str:
        return f"{BASE_RAW_URL}/{self.filename}"

    @property
    def resource_id(self) -> str:
        return f"opencc:{self.filename}"


DICTIONARIES = {
    "STPhrases.txt": DictionarySpec(
        "STPhrases.txt", "zh-CN", "zh-Hant", "opencc_st_phrase", "script", 300_000
    ),
    "STCharacters.txt": DictionarySpec(
        "STCharacters.txt", "zh-CN", "zh-Hant", "opencc_st_character", "script", 200_000
    ),
    "HKPhrases.txt": DictionarySpec(
        "HKPhrases.txt", "zh-Hant", "zh-HK", "opencc_hk_phrase", "regional", 500_000
    ),
    "HKVariantsPhrases.txt": DictionarySpec(
        "HKVariantsPhrases.txt", "zh-Hant", "zh-HK", "opencc_hk_variant_phrase", "regional", 400_000
    ),
    "HKVariants.txt": DictionarySpec(
        "HKVariants.txt", "zh-Hant", "zh-HK", "opencc_hk_variant_character", "regional", 300_000
    ),
    "TWPhrases.txt": DictionarySpec(
        "TWPhrases.txt", "zh-Hant", "zh-TW", "opencc_tw_phrase", "regional", 500_000
    ),
    "TWVariantsPhrases.txt": DictionarySpec(
        "TWVariantsPhrases.txt", "zh-Hant", "zh-TW", "opencc_tw_variant_phrase", "regional", 400_000
    ),
    "TWVariants.txt": DictionarySpec(
        "TWVariants.txt", "zh-Hant", "zh-TW", "opencc_tw_variant_character", "regional", 300_000
    ),
}


class OpenCCFormatError(ValueError):
    """Raised when an input dictionary does not match reviewed OpenCC format."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_pinned(spec: DictionarySpec, destination: Path) -> Path:
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        spec.pinned_url,
        headers={"User-Agent": "chinese-regional-localizer/phase-2c"},
    )
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("wb") as output:
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
    return destination


def iter_entries(path: Path, expected_filename: str) -> Iterable[tuple[int, str, list[str]]]:
    required_markers = {
        "# Open Chinese Convert (OpenCC) Dictionary",
        f"# File: {expected_filename}",
        "# Format: key\tvalue(s) (values separated by spaces)",
        "# License: Apache-2.0 (see LICENSE)",
    }
    seen_markers: set[str] = set()

    with path.open("r", encoding="utf-8-sig") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.rstrip("\r\n")
            if line.startswith("#"):
                if line in required_markers:
                    seen_markers.add(line)
                continue
            if not line.strip():
                continue
            if seen_markers != required_markers:
                missing = sorted(required_markers - seen_markers)
                raise OpenCCFormatError(
                    f"{expected_filename}: dictionary data started before required header markers; missing {missing}"
                )
            if line.count("\t") != 1:
                raise OpenCCFormatError(
                    f"{expected_filename}:{line_number}: expected exactly one tab separator"
                )
            source_text, raw_targets = line.split("\t", 1)
            if not source_text:
                raise OpenCCFormatError(f"{expected_filename}:{line_number}: empty source key")
            targets = [item for item in raw_targets.split(" ") if item]
            if not targets:
                raise OpenCCFormatError(f"{expected_filename}:{line_number}: no target candidates")
            yield line_number, source_text, targets

    if seen_markers != required_markers:
        missing = sorted(required_markers - seen_markers)
        raise OpenCCFormatError(f"{expected_filename}: missing required header markers {missing}")


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
    poc_builder.ensure_hardening_schema(conn, schema_path)
    poc_builder.load_manifest_sources(conn, manifest)
    return conn


def _current_version_ids_for_resource(
    conn: sqlite3.Connection, spec: DictionarySpec
) -> list[int]:
    rows = conn.execute(
        """
        SELECT source_version_id
        FROM source_versions
        WHERE source_id = ? AND is_current = 1
          AND (
              resource_key = ?
              OR (
                  resource_key = ''
                  AND (upstream_url LIKE ? OR notes LIKE ?)
              )
          )
        """,
        (
            SOURCE_ID,
            spec.resource_id,
            f"%/{spec.filename}",
            f"%{spec.filename}%",
        ),
    ).fetchall()
    return [int(row[0]) for row in rows]


def _deactivate_versions(conn: sqlite3.Connection, version_ids: list[int]) -> None:
    if not version_ids:
        return
    placeholders = ",".join("?" for _ in version_ids)
    conn.execute(
        f"UPDATE term_rules SET active = 0 WHERE source_version_id IN ({placeholders})",
        version_ids,
    )
    conn.execute(
        f"UPDATE source_versions SET is_current = 0 WHERE source_version_id IN ({placeholders})",
        version_ids,
    )


def add_source_version(
    conn: sqlite3.Connection,
    spec: DictionarySpec,
    checksum: str,
    retrieved_at: str,
    upstream_url: str,
) -> tuple[int, bool]:
    existing = poc_builder.find_current_source_version(
        conn,
        source_id=SOURCE_ID,
        resource_key=spec.resource_id,
        revision_id=PINNED_COMMIT,
        checksum_sha256=checksum,
    )
    if existing is not None:
        return existing, False

    _deactivate_versions(conn, _current_version_ids_for_resource(conn, spec))
    return poc_builder.register_source_version(
        conn,
        source_id=SOURCE_ID,
        resource_key=spec.resource_id,
        version_label=f"{PINNED_COMMIT[:12]}:{spec.filename}",
        revision_id=PINNED_COMMIT,
        published_at=None,
        retrieved_at=retrieved_at,
        upstream_url=upstream_url,
        checksum_sha256=checksum,
        notes=(
            f"OpenCC {spec.filename}; stage={spec.stage}; "
            f"locales={spec.source_locale}->{spec.target_locale}; "
            f"dictionary_base_priority={spec.base_priority}"
        ),
    )


def import_dictionary(
    conn: sqlite3.Connection,
    *,
    spec: DictionarySpec,
    input_path: Path,
    retrieved_at: str,
    upstream_url: str,
) -> dict[str, Any]:
    entries = list(iter_entries(input_path, spec.filename))
    checksum = sha256_file(input_path)
    source_version_id, created = add_source_version(
        conn, spec, checksum=checksum, retrieved_at=retrieved_at, upstream_url=upstream_url
    )
    keys = len(entries)
    candidates = sum(len(targets) for _, _, targets in entries)

    if not created:
        active_count = int(
            conn.execute(
                "SELECT COUNT(*) FROM term_rules WHERE source_version_id = ? AND active = 1",
                (source_version_id,),
            ).fetchone()[0]
        )
        if active_count != candidates:
            raise RuntimeError(
                f"Current {spec.filename} snapshot is incomplete: expected {candidates} active rules, found {active_count}"
            )
        return {
            "filename": spec.filename,
            "source_version_id": source_version_id,
            "sha256": checksum,
            "base_priority": spec.base_priority,
            "keys": keys,
            "candidates": candidates,
            "changed": False,
        }

    for line_number, source_text, targets in entries:
        for rank, target_text in enumerate(targets, start=1):
            priority = spec.base_priority - rank
            metadata = json.dumps(
                {
                    "dictionary": spec.filename,
                    "stage": spec.stage,
                    "dictionary_base_priority": spec.base_priority,
                    "candidate_rank": rank,
                    "candidate_count": len(targets),
                },
                ensure_ascii=False,
                sort_keys=True,
            )
            conn.execute(
                """
                INSERT INTO term_rules (
                    source_locale, target_locale, source_text, target_text,
                    domain, rule_type, priority, context_constraint,
                    source_id, source_version_id, upstream_record_id,
                    upstream_url, confidence, active
                ) VALUES (?, ?, ?, ?, 'opencc', ?, ?, ?, ?, ?, ?, ?, 1.0, 1)
                """,
                (
                    spec.source_locale,
                    spec.target_locale,
                    source_text,
                    target_text,
                    spec.rule_type,
                    priority,
                    metadata,
                    SOURCE_ID,
                    source_version_id,
                    f"{spec.filename}:{line_number}",
                    upstream_url,
                ),
            )

    return {
        "filename": spec.filename,
        "source_version_id": source_version_id,
        "sha256": checksum,
        "base_priority": spec.base_priority,
        "keys": keys,
        "candidates": candidates,
        "changed": True,
    }


def import_many(
    *,
    inputs: dict[str, Path],
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

    unknown = sorted(set(inputs) - set(DICTIONARIES))
    if unknown:
        raise ValueError(f"Unsupported OpenCC dictionaries: {unknown}")
    for filename in inputs:
        poc_builder.assert_source_resource_allowed(
            manifest, SOURCE_ID, DICTIONARIES[filename].resource_id
        )

    conn = open_or_create_database(db_path, schema_path, manifest, reset=reset)
    results: list[dict[str, Any]] = []
    try:
        poc_builder.assert_database_pack_compatible(conn, source["pack"])
        for filename, input_path in inputs.items():
            spec = DICTIONARIES[filename]
            upstream_url = f"fixture://opencc/{filename}" if fixture_mode else spec.pinned_url
            results.append(
                import_dictionary(
                    conn,
                    spec=spec,
                    input_path=input_path,
                    retrieved_at=retrieved_at,
                    upstream_url=upstream_url,
                )
            )
        conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('opencc_revision', ?)",
            (PINNED_COMMIT,),
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
        "revision_id": PINNED_COMMIT,
        "dictionaries": results,
        "db_path": str(db_path),
    }


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument(
        "--fixture-dir",
        type=Path,
        help="Import all reviewed dictionary filenames from a local directory",
    )
    mode.add_argument(
        "--download",
        action="store_true",
        help="Download all reviewed commit-pinned upstream dictionaries before importing",
    )
    parser.add_argument("--db", type=Path, default=repo_root / "build" / "opencc.sqlite")
    parser.add_argument("--reset", action="store_true")
    parser.add_argument("--retrieved-at", default=None)
    args = parser.parse_args()

    if args.download:
        download_dir = repo_root / "data" / "downloads" / "opencc" / PINNED_COMMIT
        inputs = {}
        for filename, spec in DICTIONARIES.items():
            destination = download_dir / filename
            download_pinned(spec, destination)
            inputs[filename] = destination
        fixture_mode = False
    else:
        inputs = {filename: args.fixture_dir / filename for filename in DICTIONARIES}
        fixture_mode = True

    result = import_many(
        inputs=inputs,
        db_path=args.db,
        manifest_path=repo_root / "data-registry" / "sources.yaml",
        schema_path=repo_root / "schema" / "sqlite-v0.2.sql",
        retrieved_at=args.retrieved_at or poc_builder.utc_now_iso(),
        reset=args.reset,
        fixture_mode=fixture_mode,
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
