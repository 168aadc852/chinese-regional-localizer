import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import import_lshk_jyutping as lshk  # noqa: E402


class LshkImporterTests(unittest.TestCase):
    def setUp(self):
        self.manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        self.schema = REPO_ROOT / "schema" / "sqlite-v0.1.sql"
        self.migration = REPO_ROOT / "schema" / "migrations" / "0002_pronunciations.sql"
        self.fixture = REPO_ROOT / "data" / "fixtures" / "lshk_sample.tsv"

    def test_fixture_import_supports_multiple_readings_and_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "lshk.sqlite"
            result = lshk.import_tsv(
                input_path=self.fixture,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                migration_path=self.migration,
                revision_id="test-fixture-revision",
                upstream_url="fixture://lshk_sample.tsv",
                retrieved_at="2026-10-06T00:00:00Z",
                reset=True,
            )

            self.assertEqual(result["source_id"], "lshk-jyutping-table")
            self.assertEqual(result["characters"], 4)
            self.assertEqual(result["readings_inserted"], 5)
            self.assertEqual(len(result["sha256"]), 64)

            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                readings = [
                    row["reading"]
                    for row in conn.execute(
                        """
                        SELECT p.reading
                        FROM pronunciations p
                        JOIN concepts c ON c.concept_id = p.concept_id
                        WHERE c.canonical_key = 'unicode:U+3405'
                        ORDER BY p.reading
                        """
                    )
                ]
                self.assertEqual(readings, ["m5", "ng5"])

                source_version = conn.execute(
                    """
                    SELECT source_id, revision_id, upstream_url, checksum_sha256
                    FROM source_versions
                    WHERE source_id = 'lshk-jyutping-table'
                    """
                ).fetchone()
                self.assertEqual(source_version["revision_id"], "test-fixture-revision")
                self.assertEqual(source_version["upstream_url"], "fixture://lshk_sample.tsv")
                self.assertEqual(source_version["checksum_sha256"], result["sha256"])

                pronunciation = conn.execute(
                    """
                    SELECT p.locale, p.romanization_scheme, p.reading,
                           p.initial, p.final, p.tone, p.source_id,
                           p.upstream_record_id, p.upstream_url
                    FROM pronunciations p
                    JOIN concepts c ON c.concept_id = p.concept_id
                    WHERE c.canonical_key = 'unicode:U+7684'
                    """
                ).fetchone()
                self.assertEqual(pronunciation["locale"], "yue-Hant-HK")
                self.assertEqual(pronunciation["romanization_scheme"], "Jyutping")
                self.assertEqual(pronunciation["reading"], "dik1")
                self.assertEqual(pronunciation["initial"], "d")
                self.assertEqual(pronunciation["final"], "ik")
                self.assertEqual(pronunciation["tone"], "1")
                self.assertEqual(pronunciation["source_id"], "lshk-jyutping-table")
                self.assertEqual(pronunciation["upstream_record_id"], "U+7684:dik1")

                self.assertEqual(conn.execute("PRAGMA integrity_check").fetchone()[0], "ok")
            finally:
                conn.close()

    def test_malformed_header_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad_tsv = Path(tmp) / "bad.tsv"
            bad_tsv.write_text("CH\tUCODE\tWRONG\n的\tU+7684\tdik1\n", encoding="utf-8")
            with self.assertRaises(lshk.LshkFormatError):
                list(lshk.iter_rows(bad_tsv))

    def test_character_ucode_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad_tsv = Path(tmp) / "bad.tsv"
            bad_tsv.write_text(
                "CH\tUCODE\tJP\tINIT\tFINL\tTONE\tDESC\tDESC_JP\n"
                "的\tU+58EB\tdik1\td\tik\t1\t\t\n",
                encoding="utf-8",
            )
            with self.assertRaises(lshk.LshkFormatError):
                list(lshk.iter_rows(bad_tsv))


if __name__ == "__main__":
    unittest.main()
