import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
SRC_DIR = REPO_ROOT / "src"
for path in (SCRIPTS_DIR, SRC_DIR):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from benchmark_localizer import benchmark, threshold_failures  # noqa: E402
from build_demo_database import build_demo  # noqa: E402
from evaluate_corpus import evaluate, load_corpus  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402


class Phase2EEvaluationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.db_path = Path(self.tmp.name) / "phase2e.sqlite"
        build_demo(self.db_path)
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.engine = LocalizerEngine(self.conn)
        self.corpus_path = REPO_ROOT / "data" / "fixtures" / "realistic_corpus.json"

    def tearDown(self):
        self.conn.close()
        self.tmp.cleanup()

    def test_realistic_corpus_is_project_authored_and_passes(self):
        corpus = load_corpus(self.corpus_path)
        self.assertGreaterEqual(len(corpus), 8)
        summary = evaluate(self.engine, corpus)
        self.assertEqual(summary["passed"], summary["total"], summary["failures"])
        self.assertEqual(summary["pass_rate"], 1.0)

    def test_short_common_entity_surface_is_not_forced(self):
        cursor = self.conn.execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('company', 'test:apple-company', 'entity')"
        )
        concept_id = int(cursor.lastrowid)
        self.conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, 'zh-CN', '苹果', 'preferred', 'entity', 1, 1.0)
            """,
            (concept_id,),
        )
        self.conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, 'zh-HK', 'Apple公司', 'preferred', 'entity', 1, 1.0)
            """,
            (concept_id,),
        )
        self.conn.commit()
        self.engine.clear_cache()

        result = self.engine.localize("我買苹果", "zh-CN", "zh-HK")
        entity_events = [event for event in result["changes"] if event["type"] == "entity"]
        self.assertEqual(entity_events, [])
        self.assertNotIn("Apple公司", result["output"])

    def test_benchmark_is_deterministic_and_reports_metrics(self):
        text = "布拉德·皮特研究人工智能。" * 40
        result = benchmark(
            self.engine,
            text=text,
            source_locale="zh-CN",
            target_locale="zh-TW",
            warmups=1,
            iterations=3,
        )
        self.assertTrue(result["deterministic"])
        self.assertGreater(result["chars_per_sec"], 0)
        self.assertGreater(result["latency_ms"]["p50"], 0)
        self.assertGreater(result["peak_python_kib"], 0)

    def test_threshold_helper_flags_catastrophic_regression(self):
        sample = {
            "deterministic": True,
            "chars_per_sec": 50.0,
            "latency_ms": {"p95": 2000.0},
            "peak_python_kib": 100000.0,
        }
        failures = threshold_failures(
            sample,
            min_chars_per_sec=100.0,
            max_p95_ms=1000.0,
            max_peak_kib=50000.0,
        )
        self.assertEqual(len(failures), 3)


if __name__ == "__main__":
    unittest.main()
