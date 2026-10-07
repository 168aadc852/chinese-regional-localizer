#!/usr/bin/env python3
"""Build deterministic unsigned release-catalog.json from validated data packages.

The resulting raw catalog bytes are intended to be signed by the Rust
`sign_metadata --kind catalog` utility. Signing is deliberately separate from
catalog construction so production private keys never need to enter this script
or the source repository.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import time
from pathlib import Path
from typing import Any

import build_data_package
import poc_builder

CATALOG_VERSION = 1


class CatalogBuildError(ValueError):
    pass


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _load_validated_package(
    package_dir: Path, source_manifest: dict[str, Any]
) -> dict[str, Any]:
    manifest_path = package_dir / "package.json"
    if not manifest_path.is_file():
        raise CatalogBuildError(f"Missing package.json: {package_dir}")
    raw_manifest = manifest_path.read_bytes()
    try:
        package = json.loads(raw_manifest)
    except json.JSONDecodeError as exc:
        raise CatalogBuildError(f"Invalid package.json in {package_dir}: {exc}") from exc

    if package.get("manifest_version") != build_data_package.PACKAGE_MANIFEST_VERSION:
        raise CatalogBuildError(f"Unsupported package manifest in {package_dir}")
    database = package.get("database")
    if not isinstance(database, dict):
        raise CatalogBuildError(f"Missing database object in {manifest_path}")
    database_file = database.get("file")
    if database_file != build_data_package.PACKAGE_DB_NAME:
        raise CatalogBuildError(f"Unexpected database filename in {manifest_path}")
    database_path = package_dir / database_file
    if not database_path.is_file():
        raise CatalogBuildError(f"Package database is missing: {database_path}")
    if database.get("size_bytes") != database_path.stat().st_size:
        raise CatalogBuildError(f"Package database size mismatch: {database_path}")
    if database.get("sha256") != build_data_package.sha256_file(database_path):
        raise CatalogBuildError(f"Package database checksum mismatch: {database_path}")

    actual_pack, _ = build_data_package.validate_database_for_release(
        database_path, source_manifest
    )
    if package.get("pack_type") != actual_pack:
        raise CatalogBuildError(
            f"Package manifest/database pack mismatch in {package_dir}"
        )

    package_id = build_data_package._safe_token(str(package.get("package_id", "")), "package_id")
    version = build_data_package._safe_token(str(package.get("version", "")), "version")
    min_runtime_api = build_data_package._safe_token(
        str(package.get("min_runtime_api", "")), "min_runtime_api"
    )
    if not min_runtime_api.isdigit():
        raise CatalogBuildError("min_runtime_api must be numeric")

    return {
        "package_id": package_id,
        "version": version,
        "pack_type": actual_pack,
        "min_runtime_api": min_runtime_api,
        "manifest_sha256": sha256_bytes(raw_manifest),
    }


def build_catalog(
    *,
    package_dirs: list[Path],
    manifest_path: Path,
    output_path: Path,
    sequence: int,
    expires_at_unix: int,
    generated_at_unix: int | None = None,
) -> dict[str, Any]:
    if sequence <= 0:
        raise CatalogBuildError("sequence must be positive")
    generated_at_unix = int(time.time()) if generated_at_unix is None else generated_at_unix
    if expires_at_unix <= generated_at_unix:
        raise CatalogBuildError("expires_at_unix must be later than generated_at_unix")
    if not package_dirs:
        raise CatalogBuildError("at least one --package-dir is required")

    source_manifest = poc_builder.load_manifest(manifest_path)
    packages = [
        _load_validated_package(package_dir, source_manifest)
        for package_dir in package_dirs
    ]
    packages.sort(key=lambda item: (item["package_id"], item["version"]))
    identities = [(item["package_id"], item["version"]) for item in packages]
    if len(identities) != len(set(identities)):
        raise CatalogBuildError("duplicate package_id/version in catalog input")

    catalog = {
        "catalog_version": CATALOG_VERSION,
        "sequence": sequence,
        "generated_at_unix": generated_at_unix,
        "expires_at_unix": expires_at_unix,
        "packages": packages,
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(catalog, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return catalog


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package-dir", type=Path, action="append", required=True)
    parser.add_argument("--manifest", type=Path, default=Path("data-registry/sources.yaml"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--sequence", type=int, required=True)
    parser.add_argument("--generated-at-unix", type=int)
    parser.add_argument("--expires-at-unix", type=int, required=True)
    args = parser.parse_args()
    catalog = build_catalog(
        package_dirs=args.package_dir,
        manifest_path=args.manifest,
        output_path=args.output,
        sequence=args.sequence,
        generated_at_unix=args.generated_at_unix,
        expires_at_unix=args.expires_at_unix,
    )
    print(json.dumps(catalog, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
