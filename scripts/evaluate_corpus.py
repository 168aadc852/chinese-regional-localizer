#!/usr/bin/env python3
"""Evaluate exact localization behavior against a synthetic corpus."""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / "src"
SCRIPTS_DIR = REPO_ROOT / "scripts"
for path in (SRC_DIR, SCRIPTS_DIR):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from build_demo_database import build_demo  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402


def load_corpus(path: Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list) or not data:
        raise ValueError("Corpus must be a non-empty JSON array")
    required = {"id", "source_locale", "target_locale", "input", "expected"}
    for index, case in enumerate(data):
        if not isinstance(case, dict):
            raise ValueError(f"Corpus item {index} must be an object")
        missing = required - set(case)
        if missing:
            raise ValueError(f"Corpus item {index} missing: {sorted(missing)}")
        if case.get("provenance") != "project-authored synthetic text":
            raise ValueError(
                f"Corpus item {case['id']} must explicitly identify project-authored provenance"
            )
    return data


def evaluate(engine: LocalizerEngine, corpus: list[dict]) -> dict:
    failures: list[dict] = []
    category_totals: dict[str, dict[str, int]] = {}
    for case in corpus:
        result = engine.localize(
            case["input"], case["source_locale"], case["target_locale"]
        )
        expected_review = bool(case.get("expected_review_needed", False))
        passed = (
            result["output"] == case["expected"]
            and bool(result["review_needed"]) == expected_review
        )
        category = case.get("category", "uncategorized")
        bucket = category_totals.setdefault(category, {"passed": 0, "total": 0})
        bucket["total"] += 1
        if passed:
            bucket["passed"] += 1
        else:
            failures.append(
                {
                    "id": case["id"],
                    "category": category,
                    "expected": case["expected"],
                    "actual": result["output"],
                    "expected_review_needed": expected_review,
                    "actual_review_needed": bool(result["review_needed"]),
                    "changes": result["changes"],
                }
            )
    passed_count = len(corpus) - len(failures)
    return {
        "passed": passed_count,
        "total": len(corpus),
        "pass_rate": passed_count / len(corpus),
        "categories": category_totals,
        "failures": failures,
    }


def evaluate_with_db(db_path: Path, corpus_path: Path) -> dict:
    corpus = load_corpus(corpus_path)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    try:
        return evaluate(LocalizerEngine(conn), corpus)
    finally:
        conn.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--corpus",
        type=Path,
        default=REPO_ROOT / "data" / "fixtures" / "realistic_corpus.json",
    )
    parser.add_argument("--db", type=Path, help="Use an existing fixture-compatible database")
    args = parser.parse_args()

    if args.db:
        summary = evaluate_with_db(args.db, args.corpus)
    else:
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "evaluation.sqlite"
            build_demo(db_path)
            summary = evaluate_with_db(db_path, args.corpus)

    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0 if summary["passed"] == summary["total"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
