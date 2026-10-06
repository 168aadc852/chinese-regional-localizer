#!/usr/bin/env python3
"""Build the offline Phase 2C demo database from repository fixtures."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import import_opencc_dictionaries as opencc_importer
import import_wikidata_entities as wikidata_importer


def build_demo(db_path: Path) -> dict:
    repo_root = Path(__file__).resolve().parents[1]
    manifest = repo_root / "data-registry" / "sources.yaml"
    schema = repo_root / "schema" / "sqlite-v0.2.sql"
    retrieved_at = "2026-10-07T00:00:00Z"

    opencc = opencc_importer.import_many(
        inputs={
            filename: repo_root / "data" / "fixtures" / "opencc" / filename
            for filename in opencc_importer.DICTIONARIES
        },
        db_path=db_path,
        manifest_path=manifest,
        schema_path=schema,
        retrieved_at=retrieved_at,
        reset=True,
        fixture_mode=True,
    )

    wikidata = wikidata_importer.import_document(
        input_path=repo_root / "data" / "fixtures" / "wikidata" / "entities.json",
        type_map_path=repo_root / "data" / "fixtures" / "wikidata" / "entity_types.json",
        db_path=db_path,
        manifest_path=manifest,
        schema_path=schema,
        retrieved_at=retrieved_at,
        reset=False,
        fixture_mode=True,
    )

    return {
        "database": str(db_path),
        "warning": "Development fixture database only; not an authoritative data release.",
        "schema_version": "0.2",
        "opencc": opencc,
        "wikidata": wikidata,
    }


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--db",
        type=Path,
        default=repo_root / "build" / "regional-demo.sqlite",
    )
    args = parser.parse_args()
    args.db.parent.mkdir(parents=True, exist_ok=True)
    print(json.dumps(build_demo(args.db), ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
