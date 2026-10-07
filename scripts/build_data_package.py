#!/usr/bin/env python3
"""Build a redistributable shared-database package from an existing SQLite DB.

The builder validates source policy and licence-pack boundaries before copying
an immutable database artifact and emitting package.json.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sqlite3
from pathlib import Path
from typing import Any

import poc_builder

PACKAGE_MANIFEST_VERSION = 1
PACKAGE_DB_NAME = "regional.sqlite"
SAFE_ID = re.compile(r"^[A-Za-z0-9._-]+$")
REDISTRIBUTABLE_PACKS = {"core", "attribution", "sharealike"}


class PackageBuildError(ValueError):
    pass


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _safe_token(value: str, field: str) -> str:
    if not value or not SAFE_ID.fullmatch(value):
        raise PackageBuildError(f"{field} must contain only letters, numbers, dot, underscore or dash")
    return value


def _metadata(conn: sqlite3.Connection, key: str) -> str | None:
    row = conn.execute("SELECT value FROM build_metadata WHERE key = ?", (key,)).fetchone()
    return str(row[0]) if row else None


def validate_database_for_release(
    db_path: Path, manifest: dict[str, Any]
) -> tuple[str, list[dict[str, Any]]]:
    if not db_path.is_file():
        raise PackageBuildError(f"Database not found: {db_path}")

    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    try:
        integrity = conn.execute("PRAGMA integrity_check").fetchone()
        if not integrity or str(integrity[0]).lower() != "ok":
            raise PackageBuildError("SQLite integrity_check failed")

        pack_type = _metadata(conn, "pack_type")
        if pack_type not in REDISTRIBUTABLE_PACKS:
            raise PackageBuildError(f"Database has invalid or missing redistributable pack_type: {pack_type!r}")

        policies = poc_builder.source_map(manifest)
        rows = conn.execute(
            """
            SELECT sv.source_id, sv.resource_key, sv.version_label, sv.revision_id,
                   sv.checksum_sha256
            FROM source_versions sv
            WHERE sv.is_current = 1
            ORDER BY sv.source_id, sv.resource_key, sv.source_version_id
            """
        ).fetchall()
        if not rows:
            raise PackageBuildError("Database has no current source_versions to package")

        sources: list[dict[str, Any]] = []
        for row in rows:
            source_id = str(row["source_id"])
            policy = policies.get(source_id)
            if policy is None:
                raise PackageBuildError(f"Database references unknown source: {source_id}")
            if not policy.get("ingest_allowed") or policy.get("status") in {
                "pending_review",
                "reference_only",
                "rejected",
            }:
                raise PackageBuildError(f"Database references non-redistributable source: {source_id}")
            if policy.get("pack") != pack_type:
                raise PackageBuildError(
                    f"Licence-pack mismatch for {source_id}: database={pack_type}, manifest={policy.get('pack')}"
                )
            resource_key = str(row["resource_key"] or "")
            allowed_resources = policy.get("ingest_resources")
            if allowed_resources is not None and resource_key not in allowed_resources:
                raise PackageBuildError(
                    f"Current resource {source_id}:{resource_key} is outside ingest_resources"
                )
            sources.append(
                {
                    "source_id": source_id,
                    "resource_key": resource_key,
                    "version_label": row["version_label"],
                    "revision_id": row["revision_id"],
                    "checksum_sha256": row["checksum_sha256"],
                }
            )
        return pack_type, sources
    finally:
        conn.close()


def build_package(
    *,
    db_path: Path,
    manifest_path: Path,
    output_dir: Path,
    package_id: str,
    version: str,
    min_runtime_api: str = "1",
    created_at: str | None = None,
) -> dict[str, Any]:
    package_id = _safe_token(package_id, "package_id")
    version = _safe_token(version, "version")
    min_runtime_api = _safe_token(min_runtime_api, "min_runtime_api")
    manifest = poc_builder.load_manifest(manifest_path)
    pack_type, sources = validate_database_for_release(db_path, manifest)

    if output_dir.exists() and any(output_dir.iterdir()):
        raise PackageBuildError(f"Output directory is not empty: {output_dir}")
    output_dir.mkdir(parents=True, exist_ok=True)
    output_db = output_dir / PACKAGE_DB_NAME
    shutil.copy2(db_path, output_db)

    package = {
        "manifest_version": PACKAGE_MANIFEST_VERSION,
        "package_id": package_id,
        "version": version,
        "pack_type": pack_type,
        "min_runtime_api": min_runtime_api,
        "created_at": created_at or poc_builder.utc_now_iso(),
        "database": {
            "file": PACKAGE_DB_NAME,
            "size_bytes": output_db.stat().st_size,
            "sha256": sha256_file(output_db),
        },
        "sources": sources,
    }
    (output_dir / "package.json").write_text(
        json.dumps(package, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return package


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=Path("data-registry/sources.yaml"))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--package-id", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--min-runtime-api", default="1")
    parser.add_argument("--created-at")
    args = parser.parse_args()
    package = build_package(
        db_path=args.db,
        manifest_path=args.manifest,
        output_dir=args.output_dir,
        package_id=args.package_id,
        version=args.version,
        min_runtime_api=args.min_runtime_api,
        created_at=args.created_at,
    )
    print(json.dumps(package, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
