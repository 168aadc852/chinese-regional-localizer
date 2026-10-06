import json
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

import import_opencc_dictionaries as opencc_importer  # noqa: E402
import import_wikidata_entities as wikidata_importer  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402


class LocalizerEngineTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.db_path = Path(self.tmp.name) / "integrated.sqlite"
        manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        schema = REPO_ROOT / "schema" / "sqlite-v0.1.sql"

        opencc_importer.import_many(
            inputs={
                filename: REPO_ROOT / "data" / "fixtures" / "opencc" / filename
                for filename in opencc_importer.DICTIONARIES
            },
            db_path=self.db_path,
            manifest_path=manifest,
            schema_path=schema,
            retrieved_at="2026-10-06T00:00:00Z",
            reset=True,
            fixture_mode=True,
        )
        wikidata_importer.import_document(
            input_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entities.json",
            type_map_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entity_types.json",
            db_path=self.db_path,
            manifest_path=manifest,
            schema_path=schema,
            retrieved_at="2026-10-06T00:00:00Z",
            reset=False,
            fixture_mode=True,
        )
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.engine = LocalizerEngine(self.conn)

    def tearDown(self):
        self.conn.close()
        self.tmp.cleanup()

    def _opencc_version_id(self) -> int:
        return int(
            self.conn.execute(
                "SELECT source_version_id FROM source_versions WHERE source_id = 'opencc' LIMIT 1"
            ).fetchone()[0]
        )

    def test_evaluation_corpus(self):
        cases = json.loads(
            (REPO_ROOT / "data" / "fixtures" / "evaluation_cases.json").read_text(
                encoding="utf-8"
            )
        )
        for case in cases:
            with self.subTest(case=case["id"]):
                result = self.engine.localize(
                    case["input"], case["source_locale"], case["target_locale"]
                )
                self.assertEqual(result["output"], case["expected"])
                self.assertFalse(result["review_needed"])

    def test_entity_changes_include_qid_and_evidence(self):
        result = self.engine.localize("布拉德·皮特", "zh-CN", "zh-HK")
        entity = next(event for event in result["changes"] if event["type"] == "entity")
        self.assertTrue(entity["applied"])
        self.assertEqual(entity["qid"], "Q35332")
        self.assertEqual(entity["replacement"], "畢·彼特")
        self.assertTrue(entity["evidence"])
        evidence = entity["evidence"][0]
        self.assertEqual(evidence["source_id"], "wikidata")
        self.assertEqual(evidence["upstream_revision"], "1000001")
        self.assertEqual(evidence["retrieved_at"], "2026-10-06T00:00:00Z")

    def test_term_changes_include_provenance(self):
        result = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        term = next(event for event in result["changes"] if event["type"] == "term_rule")
        self.assertEqual(term["replacement"], "人工智慧")
        self.assertEqual(term["source_id"], "opencc")
        self.assertIsNotNone(term["revision_id"])
        self.assertTrue(term["upstream_record_id"].startswith("TWPhrases.txt:"))
        self.assertTrue(term["upstream_url"].startswith("fixture://opencc/"))

    def test_entity_replacement_is_protected_from_later_term_rules(self):
        self.conn.execute(
            """
            INSERT INTO term_rules (
                source_locale, target_locale, source_text, target_text,
                domain, rule_type, priority, source_id, source_version_id,
                upstream_record_id, upstream_url, confidence, active
            ) VALUES ('zh-Hant', 'zh-HK', '畢·彼特', '錯誤改名',
                      'test', 'test_rule', 999999, 'opencc', ?,
                      'test:entity-protection', 'fixture://test', 1.0, 1)
            """,
            (self._opencc_version_id(),),
        )
        self.conn.commit()
        result = self.engine.localize("布拉德·皮特", "zh-CN", "zh-HK")
        self.assertEqual(result["output"], "畢·彼特")
        self.assertFalse(any(event.get("replacement") == "錯誤改名" for event in result["changes"]))

    def test_longest_match_beats_shorter_higher_priority_rule(self):
        self.conn.execute(
            """
            INSERT INTO term_rules (
                source_locale, target_locale, source_text, target_text,
                domain, rule_type, priority, source_id, source_version_id,
                upstream_record_id, upstream_url, confidence, active
            ) VALUES ('zh-Hant', 'zh-TW', '人工', '人造',
                      'test', 'test_rule', 999999, 'opencc', ?,
                      'test:shorter-rule', 'fixture://test', 1.0, 1)
            """,
            (self._opencc_version_id(),),
        )
        self.conn.commit()
        result = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        self.assertEqual(result["output"], "人工智慧")

    def test_ambiguous_rule_tie_is_not_guessed(self):
        version_id = self._opencc_version_id()
        for target in ("候選甲", "候選乙"):
            self.conn.execute(
                """
                INSERT INTO term_rules (
                    source_locale, target_locale, source_text, target_text,
                    domain, rule_type, priority, source_id, source_version_id,
                    upstream_record_id, upstream_url, confidence, active
                ) VALUES ('zh-Hant', 'zh-HK', '測試詞', ?,
                          'test', 'test_rule', 500, 'opencc', ?, ?,
                          'fixture://test', 0.5, 1)
                """,
                (target, version_id, f"test:tie:{target}"),
            )
        self.conn.commit()

        result = self.engine.localize("測試詞", "zh-Hant", "zh-HK")
        self.assertEqual(result["output"], "測試詞")
        self.assertTrue(result["review_needed"])
        review = next(event for event in result["changes"] if event["review_needed"])
        self.assertEqual(review["reason"], "ambiguous_rule_tie")
        self.assertEqual(
            {candidate["target_text"] for candidate in review["candidates"]},
            {"候選甲", "候選乙"},
        )

    def test_ambiguous_entity_is_not_guessed(self):
        cursor = self.conn.execute(
            "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('person', 'test:duplicate-person', 'entity')"
        )
        concept_id = int(cursor.lastrowid)
        self.conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, 'zh-CN', '布拉德·皮特', 'preferred', 'entity', 1, 1.0)
            """,
            (concept_id,),
        )
        self.conn.execute(
            """
            INSERT INTO localized_names (
                concept_id, locale, text, name_type, domain, is_preferred, confidence
            ) VALUES (?, 'zh-HK', '另一個人', 'preferred', 'entity', 1, 1.0)
            """,
            (concept_id,),
        )
        self.conn.commit()

        result = self.engine.localize("布拉德·皮特", "zh-CN", "zh-HK")
        self.assertEqual(result["output"], "布拉德·皮特")
        self.assertTrue(result["review_needed"])
        review = next(event for event in result["changes"] if event["review_needed"])
        self.assertEqual(review["reason"], "ambiguous_source_entity")
        self.assertEqual(len(review["candidates"]), 2)


if __name__ == "__main__":
    unittest.main()
