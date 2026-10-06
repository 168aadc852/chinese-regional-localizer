#!/usr/bin/env python3
"""Import selected OpenCC phrase dictionaries into the project SQLite schema.

Phase 1B intentionally models OpenCC as staged conversion data rather than
flattening HK/TW localisation into a single global replacement table.
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

    @property
    def pinned_url(self) -> str:
        return f"{BASE_RAW_URL}/{self.filename}"


DICTIONARIES = {
    "STPhrases.txt": DictionarySpec(
        "STPhrases.txt", "zh-CN", "zh-Hant", "opencc_st_phrase", "script"
    ),
    "HKPhrases.txt": DictionarySpec(
        "HKPhrases.txt", "zh-Hant", "zh-HK", "opencc_hk_phrase", "regional"
    ),
    "TWPhrases.txt": DictionarySpec(
        "TWPhrases.txt", "zh-Hant", "zh-TW", "opencc_tw_phrase", "regional"
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
        headers={"User-Agent": "chinese-regional-localizer/phase-1b"},
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
        poc_builder.load_manifest_sources(conn, manifest)
    return conn


def add_source_version(
    conn: sqlite3.Connection,
    spec: DictionarySpec,
    checksum: str,
    retrieved_at: str,
    upstream_url: str,
) -> int:
    cursor = conn.execute(
        """
        INSERT INTO source_versions (
            source_id, version_label, revision_id, retrieved_at,
            upstream_url, checksum_sha256, notes
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            SOURCE_ID,
            f"{PINNED_COMMIT[:12]}:{spec.filename}",
            PINNED_COMMIT,
            retrieved_at,
            upstream_url,
            checksum,
            f"OpenCC {spec.filename}; stage={spec.stage}; locales={spec.source_locale}->{spec.target_locale}",
        ),
    )
    return int(cursor.lastrowid)


def import_dictionary(
    conn: sqlite3.Connection,
    *,
    spec: DictionarySpec,
    input_path: Path,
    retrieved_at: str,
    upstream_url: str,
) -> dict[str, Any]:
    checksum = sha256_file(input_path)
    source_version_id = add_source_version(
        conn, spec, checksum=checksum, retrieved_at=retrieved_at, upstream_url=upstream_url
    )
    keys = 0
    candidates = 0

    for line_number, source_text, targets in iter_entries(input_path, spec.filename):
        keys += 1
        for rank, target_text in enumerate(targets, start=1):
            candidates += 1
            context = json.dumps(
                {
                    "dictionary": spec.filename,
                    "stage": spec.stage,
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
                    10000 - rank,
                    context,
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
        "keys": keys,
        "candidates": candidates,
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

    conn = open_or_create_database(db_path, schema_path, manifest, reset=reset)
    results: list[dict[str, Any]] = []
    try:
        for filename, input_path in inputs.items():
            spec = DICTIONARIES[filename]
            upstream_url = (
                f"fixture://opencc/{filename}" if fixture_mode else spec.pinned_url
            )
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
        help="Import the three reviewed dictionary filenames from a local directory",
    )
    mode.add_argument(
        "--download",
        action="store_true",
        help="Download the three commit-pinned upstream dictionaries before importing",
    )
    parser.add_argument("--db", type=Path, default=repo_root / "build" / "opencc.sqlite")
    parser.add_argument("--reset", action="store_true")
    parser.add_argument("--retrieved-at", default="2026-10-06T00:00:00Z")
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
        schema_path=repo_root / "schema" / "sqlite-v0.1.sql",
        retrieved_at=args.retrieved_at,
        reset=args.reset,
        fixture_mode=fixture_mode,
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
