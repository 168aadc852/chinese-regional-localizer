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

import import_wikidata_entities as wikidata_importer  # noqa: E402


class WikidataImporterTests(unittest.TestCase):
    def setUp(self):
        self.manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        self.schema = REPO_ROOT / "schema" / "sqlite-v0.1.sql"
        self.fixture = REPO_ROOT / "data" / "fixtures" / "wikidata" / "entities.json"
        self.type_map = REPO_ROOT / "data" / "fixtures" / "wikidata" / "entity_types.json"

    def test_import_keeps_regional_names_separate_with_qid_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "wikidata.sqlite"
            result = wikidata_importer.import_document(
                input_path=self.fixture,
                type_map_path=self.type_map,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-06T00:00:00Z",
                reset=True,
                fixture_mode=True,
            )
            self.assertEqual(result["source_id"], "wikidata")
            self.assertEqual(result["entities"], 2)
            self.assertGreaterEqual(result["names"], 10)

            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                pitt = conn.execute(
                    """
                    SELECT c.concept_type, e.external_value
                    FROM concepts c
                    JOIN external_ids e ON e.concept_id = c.concept_id
                    WHERE e.namespace = 'wikidata' AND e.external_value = 'Q35332'
                    """
                ).fetchone()
                self.assertEqual(pitt["concept_type"], "person")
                self.assertEqual(pitt["external_value"], "Q35332")

                regional = conn.execute(
                    """
                    SELECT ln.locale, ln.text
                    FROM localized_names ln
                    JOIN external_ids e ON e.concept_id = ln.concept_id
                    WHERE e.namespace = 'wikidata'
                      AND e.external_value = 'Q35332'
                      AND ln.name_type = 'preferred'
                      AND ln.locale IN ('zh-CN','zh-HK','zh-TW')
                    ORDER BY ln.locale
                    """
                ).fetchall()
                self.assertEqual(
                    [(row["locale"], row["text"]) for row in regional],
                    [
                        ("zh-CN", "布拉德·皮特"),
                        ("zh-HK", "畢·彼特"),
                        ("zh-TW", "布萊德·彼特"),
                    ],
                )

                film_names = conn.execute(
                    """
                    SELECT ln.locale, ln.text
                    FROM localized_names ln
                    JOIN external_ids e ON e.concept_id = ln.concept_id
                    WHERE e.namespace = 'wikidata'
                      AND e.external_value = 'Q108839994'
                      AND ln.name_type = 'preferred'
                      AND ln.locale IN ('zh-CN','zh-HK','zh-TW')
                    ORDER BY ln.locale
                    """
                ).fetchall()
                self.assertEqual(
                    [(row["locale"], row["text"]) for row in film_names],
                    [
                        ("zh-CN", "奥本海默"),
                        ("zh-HK", "奧本海默"),
                        ("zh-TW", "奧本海默"),
                    ],
                )

                alias = conn.execute(
                    """
                    SELECT ln.locale, ln.text, ne.evidence_type, ne.upstream_revision, ne.upstream_url
                    FROM localized_names ln
                    JOIN external_ids e ON e.concept_id = ln.concept_id
                    JOIN name_evidence ne ON ne.localized_name_id = ln.localized_name_id
                    WHERE e.external_value = 'Q35332'
                      AND ln.name_type = 'alias'
                      AND ln.locale = 'zh-HK'
                    """
                ).fetchone()
                self.assertEqual(alias["text"], "畢彼特")
                self.assertEqual(alias["evidence_type"], "alias")
                self.assertEqual(alias["upstream_revision"], "1000001")
                self.assertEqual(alias["upstream_url"], "fixture://wikidata/Q35332.json")

                versions = conn.execute(
                    """
                    SELECT version_label, revision_id, checksum_sha256, upstream_url
                    FROM source_versions
                    WHERE source_id = 'wikidata'
                    ORDER BY source_version_id
                    """
                ).fetchall()
                self.assertEqual(len(versions), 2)
                self.assertTrue(all(len(row["checksum_sha256"]) == 64 for row in versions))
                self.assertEqual(conn.execute("PRAGMA integrity_check").fetchone()[0], "ok")
            finally:
                conn.close()

    def test_missing_regional_label_is_not_synthesized(self):
        entity = {
            "id": "Q42",
            "type": "item",
            "lastrevid": 123,
            "labels": {
                "zh": {"language": "zh", "value": "道格拉斯·亞當斯"},
                "en": {"language": "en", "value": "Douglas Adams"},
            },
            "aliases": {},
        }
        names = list(wikidata_importer.iter_names(entity))
        locales = [item[0] for item in names]
        self.assertIn("zh", locales)
        self.assertNotIn("zh-HK", locales)
        self.assertNotIn("zh-TW", locales)
        self.assertNotIn("zh-CN", locales)

    def test_entity_key_id_mismatch_is_rejected(self):
        bad = {
            "entities": {
                "Q35332": {
                    "id": "Q123",
                    "type": "item",
                    "lastrevid": 1,
                    "labels": {},
                    "aliases": {},
                }
            }
        }
        with self.assertRaises(wikidata_importer.WikidataFormatError):
            list(wikidata_importer.iter_entities(bad))

    def test_missing_type_map_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad_map = Path(tmp) / "types.json"
            bad_map.write_text(json.dumps({"Q35332": "person"}), encoding="utf-8")
            with self.assertRaises(ValueError):
                wikidata_importer.import_document(
                    input_path=self.fixture,
                    type_map_path=bad_map,
                    db_path=Path(tmp) / "db.sqlite",
                    manifest_path=self.manifest,
                    schema_path=self.schema,
                    retrieved_at="2026-10-06T00:00:00Z",
                    reset=True,
                    fixture_mode=True,
                )


if __name__ == "__main__":
    unittest.main()
