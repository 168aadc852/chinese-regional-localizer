#!/usr/bin/env python3
"""Measure reference-engine throughput, latency and tracked Python memory."""

from __future__ import annotations

import argparse
import json
import math
import sqlite3
import statistics
import sys
import tempfile
import time
import tracemalloc
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / "src"
SCRIPTS_DIR = REPO_ROOT / "scripts"
for path in (SRC_DIR, SCRIPTS_DIR):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from build_demo_database import build_demo  # noqa: E402
from evaluate_corpus import load_corpus  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(p * len(ordered)) - 1))
    return ordered[index]


def build_payload(corpus: list[dict], target_chars: int) -> str:
    seed = "\n".join(case["input"] for case in corpus)
    if not seed:
        raise ValueError("Corpus produced an empty benchmark seed")
    repeats = max(1, math.ceil(target_chars / len(seed)))
    return (seed + "\n") * repeats


def benchmark(
    engine: LocalizerEngine,
    *,
    text: str,
    source_locale: str,
    target_locale: str,
    warmups: int,
    iterations: int,
) -> dict:
    for _ in range(warmups):
        engine.localize(text, source_locale, target_locale)

    durations_ms: list[float] = []
    outputs: list[str] = []
    for _ in range(iterations):
        started = time.perf_counter()
        result = engine.localize(text, source_locale, target_locale)
        durations_ms.append((time.perf_counter() - started) * 1000.0)
        outputs.append(result["output"])

    deterministic = all(output == outputs[0] for output in outputs)
    total_seconds = sum(durations_ms) / 1000.0
    chars_processed = len(text) * iterations
    chars_per_sec = chars_processed / total_seconds if total_seconds else float("inf")

    tracemalloc.start()
    engine.localize(text, source_locale, target_locale)
    _current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()

    return {
        "source_locale": source_locale,
        "target_locale": target_locale,
        "input_chars": len(text),
        "warmups": warmups,
        "iterations": iterations,
        "deterministic": deterministic,
        "chars_per_sec": chars_per_sec,
        "latency_ms": {
            "mean": statistics.fmean(durations_ms),
            "p50": percentile(durations_ms, 0.50),
            "p95": percentile(durations_ms, 0.95),
            "max": max(durations_ms),
        },
        "peak_python_kib": peak / 1024.0,
    }


def threshold_failures(
    result: dict,
    *,
    min_chars_per_sec: float,
    max_p95_ms: float,
    max_peak_kib: float,
) -> list[str]:
    failures: list[str] = []
    if not result["deterministic"]:
        failures.append("output was not deterministic across repeated runs")
    if min_chars_per_sec and result["chars_per_sec"] < min_chars_per_sec:
        failures.append(
            f"throughput {result['chars_per_sec']:.1f} < {min_chars_per_sec:.1f} chars/sec"
        )
    p95 = result["latency_ms"]["p95"]
    if max_p95_ms and p95 > max_p95_ms:
        failures.append(f"p95 latency {p95:.1f} ms > {max_p95_ms:.1f} ms")
    if max_peak_kib and result["peak_python_kib"] > max_peak_kib:
        failures.append(
            f"peak tracked memory {result['peak_python_kib']:.1f} KiB > {max_peak_kib:.1f} KiB"
        )
    return failures


def run_with_db(
    db_path: Path,
    corpus_path: Path,
    *,
    target_chars: int,
    warmups: int,
    iterations: int,
    source_locale: str,
    target_locale: str,
) -> dict:
    corpus = [
        case
        for case in load_corpus(corpus_path)
        if case["source_locale"] == source_locale and case["target_locale"] == target_locale
    ]
    if not corpus:
        raise ValueError(f"No corpus cases for {source_locale}->{target_locale}")
    text = build_payload(corpus, target_chars)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    try:
        engine = LocalizerEngine(conn)
        return benchmark(
            engine,
            text=text,
            source_locale=source_locale,
            target_locale=target_locale,
            warmups=warmups,
            iterations=iterations,
        )
    finally:
        conn.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--corpus",
        type=Path,
        default=REPO_ROOT / "data" / "fixtures" / "realistic_corpus.json",
    )
    parser.add_argument("--db", type=Path)
    parser.add_argument("--source-locale", default="zh-CN")
    parser.add_argument("--target-locale", default="zh-TW")
    parser.add_argument("--target-chars", type=int, default=20000)
    parser.add_argument("--warmups", type=int, default=3)
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--min-chars-per-sec", type=float, default=0.0)
    parser.add_argument("--max-p95-ms", type=float, default=0.0)
    parser.add_argument("--max-peak-kib", type=float, default=0.0)
    args = parser.parse_args()

    if args.target_chars <= 0 or args.iterations <= 0 or args.warmups < 0:
        parser.error("target-chars/iterations must be positive and warmups non-negative")

    if args.db:
        result = run_with_db(
            args.db,
            args.corpus,
            target_chars=args.target_chars,
            warmups=args.warmups,
            iterations=args.iterations,
            source_locale=args.source_locale,
            target_locale=args.target_locale,
        )
    else:
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "benchmark.sqlite"
            build_demo(db_path)
            result = run_with_db(
                db_path,
                args.corpus,
                target_chars=args.target_chars,
                warmups=args.warmups,
                iterations=args.iterations,
                source_locale=args.source_locale,
                target_locale=args.target_locale,
            )

    failures = threshold_failures(
        result,
        min_chars_per_sec=args.min_chars_per_sec,
        max_p95_ms=args.max_p95_ms,
        max_peak_kib=args.max_peak_kib,
    )
    result["threshold_failures"] = failures
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
