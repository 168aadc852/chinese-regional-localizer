import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import poc_builder  # noqa: E402


class PocBuilderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest_path = REPO_ROOT / "data-registry" / "sources.yaml"
        cls.schema_path = REPO_ROOT / "schema" / "sqlite-v0.1.sql"
        cls.fixture_path = REPO_ROOT / "data" / "fixtures" / "poc_records.json"
        cls.manifest = poc_builder.load_manifest(cls.manifest_path)

    def test_pending_source_is_blocked(self):
        with self.assertRaises(poc_builder.IngestPolicyError):
            poc_builder.assert_source_ingest_allowed(self.manifest, "hk-doj-glossary")

    def test_unknown_source_is_blocked(self):
        with self.assertRaises(poc_builder.IngestPolicyError):
            poc_builder.assert_source_ingest_allowed(self.manifest, "not-a-real-source")

    def test_manifest_disallows_ingest_for_pending_sources(self):
        for source in self.manifest["sources"]:
            if source["status"] in {"pending_review", "reference_only", "rejected"}:
                self.assertFalse(source["ingest_allowed"], source["id"])

    def test_build_and_query_poc_database(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "poc.sqlite"
            poc_builder.build_database(
                self.manifest_path,
                self.schema_path,
                self.fixture_path,
                db_path,
            )
            self.assertTrue(db_path.exists())

            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                self.assertEqual(conn.execute("PRAGMA integrity_check").fetchone()[0], "ok")

                expected_sources = len(self.manifest["sources"])
                source_count = conn.execute("SELECT COUNT(*) FROM sources").fetchone()[0]
                self.assertEqual(source_count, expected_sources)

                rules = {
                    row["target_locale"]: row["target_text"]
                    for row in conn.execute(
                        "SELECT target_locale, target_text FROM term_rules WHERE source_text = '软件'"
                    )
                }
                self.assertEqual(rules["zh-HK"], "軟件")
                self.assertEqual(rules["zh-TW"], "軟體")

                film_names = {
                    row["locale"]: row["text"]
                    for row in conn.execute(
                        """
                        SELECT n.locale, n.text
                        FROM concepts c
                        JOIN localized_names n ON n.concept_id = c.concept_id
                        WHERE c.canonical_key = 'poc:inside-out'
                        """
                    )
                }
                self.assertEqual(film_names["zh-HK"], "玩轉腦朋友")
                self.assertEqual(film_names["zh-TW"], "腦筋急轉彎")

                person_hk = conn.execute(
                    """
                    SELECT n.text
                    FROM concepts c
                    JOIN localized_names n ON n.concept_id = c.concept_id
                    WHERE c.canonical_key = 'poc:brad-pitt' AND n.locale = 'zh-HK'
                    """
                ).fetchone()[0]
                self.assertEqual(person_hk, "畢·彼特")

                hk_signal = conn.execute(
                    """
                    SELECT n.text
                    FROM concepts c
                    JOIN localized_names n ON n.concept_id = c.concept_id
                    WHERE c.canonical_key = 'poc:hk-taxi' AND n.locale = 'zh-HK'
                    """
                ).fetchone()[0]
                self.assertEqual(hk_signal, "的士")

                evidence = conn.execute(
                    """
                    SELECT e.source_id, e.evidence_type, e.upstream_url
                    FROM concepts c
                    JOIN localized_names n ON n.concept_id = c.concept_id
                    JOIN name_evidence e ON e.localized_name_id = n.localized_name_id
                    WHERE c.canonical_key = 'poc:inside-out' AND n.locale = 'zh-HK'
                    """
                ).fetchone()
                self.assertEqual(evidence["source_id"], "wikidata")
                self.assertEqual(evidence["evidence_type"], "poc_fixture")
                self.assertTrue(evidence["upstream_url"].startswith("fixture://"))

                authoritative = conn.execute(
                    "SELECT value FROM build_metadata WHERE key = 'fixture_data_authoritative'"
                ).fetchone()[0]
                self.assertEqual(authoritative, "false")
            finally:
                conn.close()


if __name__ == "__main__":
    unittest.main()
