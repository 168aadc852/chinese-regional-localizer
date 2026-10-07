#!/usr/bin/env python3
"""Run the deterministic localization reference engine."""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from localizer_engine import LocalizerEngine  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", type=Path, required=True, help="SQLite localization database")
    parser.add_argument("--from", dest="source_locale", required=True)
    parser.add_argument("--to", dest="target_locale", required=True)
    parser.add_argument("--text", help="Text to localize; omit to read UTF-8 from stdin")
    parser.add_argument("--domain", help="Optional runtime domain for context-constrained rules")
    args = parser.parse_args()

    text = args.text if args.text is not None else sys.stdin.read()
    conn = sqlite3.connect(args.db)
    conn.row_factory = sqlite3.Row
    try:
        result = LocalizerEngine(conn).localize(
            text=text,
            source_locale=args.source_locale,
            target_locale=args.target_locale,
            context={"domain": args.domain} if args.domain else None,
        )
    finally:
        conn.close()

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
