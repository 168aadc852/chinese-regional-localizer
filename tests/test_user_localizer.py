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
from user_dictionary import UserDictionary  # noqa: E402
from user_localizer import UserControlledLocalizer  # noqa: E402


class UserLocalizerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.shared_db = root / "shared.sqlite"
        self.user_db = root / "user_dictionary.sqlite"
        manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        schema = REPO_ROOT / "schema" / "sqlite-v0.2.sql"

        opencc_importer.import_many(
            inputs={
                filename: REPO_ROOT / "data" / "fixtures" / "opencc" / filename
                for filename in opencc_importer.DICTIONARIES
            },
            db_path=self.shared_db,
            manifest_path=manifest,
            schema_path=schema,
            retrieved_at="2026-10-07T00:00:00Z",
            reset=True,
            fixture_mode=True,
        )
        wikidata_importer.import_document(
            input_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entities.json",
            type_map_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entity_types.json",
            db_path=self.shared_db,
            manifest_path=manifest,
            schema_path=schema,
            retrieved_at="2026-10-07T00:00:00Z",
            reset=False,
            fixture_mode=True,
        )

        self.shared_conn = sqlite3.connect(self.shared_db)
        self.shared_conn.row_factory = sqlite3.Row
        self.dictionary = UserDictionary.open(
            self.user_db, REPO_ROOT / "schema" / "user-dictionary-v0.1.sql"
        )
        self.engine = UserControlledLocalizer(
            LocalizerEngine(self.shared_conn), self.dictionary
        )

    def tearDown(self):
        self.dictionary.close()
        self.shared_conn.close()
        self.tmp.cleanup()

    def test_protected_term_beats_entity(self):
        self.dictionary.add_protected("布拉德·皮特")
        result = self.engine.localize("布拉德·皮特", "zh-CN", "zh-HK")
        self.assertEqual(result["output"], "布拉德·皮特")
        event = result["changes"][0]
        self.assertEqual(event["type"], "user_protected")
        self.assertEqual(event["reason"], "protected_by_user")
        self.assertEqual(event["provenance"], "user_dictionary")

    def test_override_beats_entity(self):
        term_id = self.dictionary.add_override(
            "布拉德·皮特",
            "我指定的彼特",
            source_locale="zh-CN",
            target_locale="zh-HK",
        )
        result = self.engine.localize("布拉德·皮特", "zh-CN", "zh-HK")
        self.assertEqual(result["output"], "我指定的彼特")
        event = result["changes"][0]
        self.assertEqual(event["type"], "user_override")
        self.assertEqual(event["user_term_id"], term_id)
        self.assertEqual(event["final_output_span"], [0, len("我指定的彼特")])

    def test_protected_term_beats_opencc(self):
        self.dictionary.add_protected(
            "人工智能", source_locale="zh-CN", target_locale="zh-TW"
        )
        result = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        self.assertEqual(result["output"], "人工智能")
        self.assertFalse(any(event["type"] == "term_rule" for event in result["changes"]))

    def test_locale_scoped_override_does_not_leak(self):
        self.dictionary.add_override(
            "人工智能",
            "AI香港用詞",
            source_locale="zh-CN",
            target_locale="zh-HK",
        )
        hk = self.engine.localize("人工智能", "zh-CN", "zh-HK")
        tw = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        self.assertEqual(hk["output"], "AI香港用詞")
        self.assertEqual(tw["output"], "人工智慧")

    def test_longest_user_surface_wins(self):
        self.dictionary.add_override("人工", "人造", target_locale="zh-TW")
        self.dictionary.add_override("人工智能", "自訂AI", target_locale="zh-TW")
        result = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        self.assertEqual(result["output"], "自訂AI")

    def test_protected_beats_override_for_same_surface(self):
        self.dictionary.add_override("OpenAI", "開放人工智能", target_locale="zh-HK")
        self.dictionary.add_protected("OpenAI", target_locale="zh-HK")
        result = self.engine.localize("OpenAI", "zh-CN", "zh-HK")
        self.assertEqual(result["output"], "OpenAI")
        self.assertEqual(result["changes"][0]["type"], "user_protected")

    def test_more_specific_scope_beats_global_override(self):
        self.dictionary.add_override("測試詞", "全球答案")
        self.dictionary.add_override(
            "測試詞", "香港答案", source_locale="zh-Hant", target_locale="zh-HK"
        )
        result = self.engine.localize("測試詞", "zh-Hant", "zh-HK")
        self.assertEqual(result["output"], "香港答案")

    def test_dictionary_upsert_and_persistence(self):
        term_id = self.dictionary.add_override("軟件", "指定一", target_locale="zh-HK")
        same_id = self.dictionary.add_override("軟件", "指定二", target_locale="zh-HK")
        self.assertEqual(term_id, same_id)
        self.dictionary.close()
        self.dictionary = UserDictionary.open(
            self.user_db, REPO_ROOT / "schema" / "user-dictionary-v0.1.sql"
        )
        self.engine = UserControlledLocalizer(
            LocalizerEngine(self.shared_conn), self.dictionary
        )
        terms = self.dictionary.list_terms()
        self.assertEqual(len(terms), 1)
        self.assertEqual(terms[0].replacement, "指定二")

    def test_disable_restores_shared_behavior(self):
        term_id = self.dictionary.add_override(
            "人工智能", "停用答案", target_locale="zh-TW"
        )
        self.assertTrue(self.dictionary.set_enabled(term_id, False))
        result = self.engine.localize("人工智能", "zh-CN", "zh-TW")
        self.assertEqual(result["output"], "人工智慧")

    def test_shared_changes_keep_global_original_and_final_spans(self):
        self.dictionary.add_protected("OpenAI")
        result = self.engine.localize("OpenAI人工智能", "zh-CN", "zh-TW")
        self.assertEqual(result["output"], "OpenAI人工智慧")
        shared = next(event for event in result["changes"] if event["type"] == "term_rule")
        self.assertEqual(shared["original_input_span"], [6, 10])
        self.assertEqual(shared["final_output_span"], [6, 10])


if __name__ == "__main__":
    unittest.main()
