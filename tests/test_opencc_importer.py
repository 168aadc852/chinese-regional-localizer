import json
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import import_opencc_dictionaries as opencc_importer  # noqa: E402


class OpenCCImporterTests(unittest.TestCase):
    def setUp(self):
        self.manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        self.schema = REPO_ROOT / "schema" / "sqlite-v0.1.sql"
        self.fixture_dir = REPO_ROOT / "data" / "fixtures" / "opencc"

    def test_fixture_import_preserves_stages_candidates_and_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "opencc.sqlite"
            inputs = {
                filename: self.fixture_dir / filename
                for filename in opencc_importer.DICTIONARIES
            }
            result = opencc_importer.import_many(
                inputs=inputs,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-06T00:00:00Z",
                reset=True,
                fixture_mode=True,
            )

            self.assertEqual(result["source_id"], "opencc")
            self.assertEqual(result["revision_id"], opencc_importer.PINNED_COMMIT)
            self.assertEqual(len(result["dictionaries"]), 3)
            self.assertTrue(all(len(item["sha256"]) == 64 for item in result["dictionaries"]))

            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                brad = conn.execute(
                    """
                    SELECT source_locale, target_locale, source_text, target_text,
                           rule_type, upstream_record_id, upstream_url
                    FROM term_rules
                    WHERE source_text = '布拉德·皮特'
                    """
                ).fetchone()
                self.assertEqual(brad["source_locale"], "zh-Hant")
                self.assertEqual(brad["target_locale"], "zh-HK")
                self.assertEqual(brad["target_text"], "畢·彼特")
                self.assertEqual(brad["rule_type"], "opencc_hk_phrase")
                self.assertTrue(brad["upstream_record_id"].startswith("HKPhrases.txt:"))
                self.assertEqual(brad["upstream_url"], "fixture://opencc/HKPhrases.txt")

                ai = conn.execute(
                    """
                    SELECT source_locale, target_locale, target_text
                    FROM term_rules
                    WHERE source_text = '人工智能'
                    """
                ).fetchone()
                self.assertEqual(tuple(ai), ("zh-Hant", "zh-TW", "人工智慧"))

                st = conn.execute(
                    """
                    SELECT source_locale, target_locale, target_text
                    FROM term_rules
                    WHERE source_text = '一见钟情'
                    """
                ).fetchone()
                self.assertEqual(tuple(st), ("zh-CN", "zh-Hant", "一見鍾情"))

                candidates = conn.execute(
                    """
                    SELECT target_text, priority, context_constraint
                    FROM term_rules
                    WHERE source_text = '代碼' AND target_locale = 'zh-TW'
                    ORDER BY priority DESC
                    """
                ).fetchall()
                self.assertEqual([row["target_text"] for row in candidates], ["程式碼", "代碼"])
                contexts = [json.loads(row["context_constraint"]) for row in candidates]
                self.assertEqual([item["candidate_rank"] for item in contexts], [1, 2])
                self.assertTrue(all(item["candidate_count"] == 2 for item in contexts))

                identity = conn.execute(
                    """
                    SELECT target_text FROM term_rules
                    WHERE source_text = '一出' AND target_text = '一出'
                    """
                ).fetchone()
                self.assertIsNotNone(identity)

                versions = conn.execute(
                    """
                    SELECT revision_id, upstream_url, checksum_sha256
                    FROM source_versions
                    WHERE source_id = 'opencc'
                    ORDER BY source_version_id
                    """
                ).fetchall()
                self.assertEqual(len(versions), 3)
                self.assertTrue(all(row["revision_id"] == opencc_importer.PINNED_COMMIT for row in versions))
                self.assertTrue(all(len(row["checksum_sha256"]) == 64 for row in versions))
                self.assertEqual(conn.execute("PRAGMA integrity_check").fetchone()[0], "ok")
            finally:
                conn.close()

    def test_missing_required_header_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "HKPhrases.txt"
            path.write_text("# File: HKPhrases.txt\n服務器\t伺服器\n", encoding="utf-8")
            with self.assertRaises(opencc_importer.OpenCCFormatError):
                list(opencc_importer.iter_entries(path, "HKPhrases.txt"))

    def test_malformed_dictionary_line_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "TWPhrases.txt"
            path.write_text(
                "# Open Chinese Convert (OpenCC) Dictionary\n"
                "# File: TWPhrases.txt\n"
                "# Format: key\tvalue(s) (values separated by spaces)\n"
                "# License: Apache-2.0 (see LICENSE)\n"
                "bad line without tab\n",
                encoding="utf-8",
            )
            with self.assertRaises(opencc_importer.OpenCCFormatError):
                list(opencc_importer.iter_entries(path, "TWPhrases.txt"))


if __name__ == "__main__":
    unittest.main()
