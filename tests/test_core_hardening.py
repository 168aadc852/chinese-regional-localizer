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

import import_lshk_jyutping as lshk  # noqa: E402
import import_opencc_dictionaries as opencc  # noqa: E402
import import_wikidata_entities as wikidata  # noqa: E402
import poc_builder  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402


class CoreHardeningTests(unittest.TestCase):
    def setUp(self):
        self.manifest = REPO_ROOT / "data-registry" / "sources.yaml"
        self.schema = REPO_ROOT / "schema" / "sqlite-v0.2.sql"
        self.old_schema = REPO_ROOT / "schema" / "sqlite-v0.1.sql"
        self.opencc_inputs = {
            filename: REPO_ROOT / "data" / "fixtures" / "opencc" / filename
            for filename in opencc.DICTIONARIES
        }

    def _build_core(self, db_path: Path, schema: Path | None = None) -> None:
        opencc.import_many(
            inputs=self.opencc_inputs,
            db_path=db_path,
            manifest_path=self.manifest,
            schema_path=schema or self.schema,
            retrieved_at="2026-10-07T00:00:00Z",
            reset=True,
            fixture_mode=True,
        )
        wikidata.import_document(
            input_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entities.json",
            type_map_path=REPO_ROOT / "data" / "fixtures" / "wikidata" / "entity_types.json",
            db_path=db_path,
            manifest_path=self.manifest,
            schema_path=schema or self.schema,
            retrieved_at="2026-10-07T00:00:00Z",
            reset=False,
            fixture_mode=True,
        )

    def test_machine_readable_scope_gate_and_data_gov_hk_block(self):
        manifest = poc_builder.load_manifest(self.manifest)
        allowed = poc_builder.assert_source_resource_allowed(
            manifest, "opencc", "opencc:STPhrases.txt"
        )
        self.assertEqual(allowed["pack"], "core")
        with self.assertRaises(poc_builder.IngestPolicyError):
            poc_builder.assert_source_resource_allowed(
                manifest, "opencc", "opencc:not-reviewed.txt"
            )
        with self.assertRaises(poc_builder.IngestPolicyError):
            poc_builder.assert_source_ingest_allowed(manifest, "data-gov-hk")

    def test_opencc_reimport_same_snapshot_is_idempotent(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "opencc.sqlite"
            first = opencc.import_many(
                inputs=self.opencc_inputs,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-07T00:00:00Z",
                reset=True,
                fixture_mode=True,
            )
            second = opencc.import_many(
                inputs=self.opencc_inputs,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-07T01:00:00Z",
                reset=False,
                fixture_mode=True,
            )
            self.assertTrue(all(item["changed"] for item in first["dictionaries"]))
            self.assertTrue(all(not item["changed"] for item in second["dictionaries"]))

            conn = sqlite3.connect(db_path)
            try:
                versions = conn.execute(
                    "SELECT COUNT(*) FROM source_versions WHERE source_id = 'opencc'"
                ).fetchone()[0]
                rules = conn.execute("SELECT COUNT(*) FROM term_rules").fetchone()[0]
                active = conn.execute(
                    "SELECT COUNT(*) FROM term_rules WHERE active = 1"
                ).fetchone()[0]
                self.assertEqual(versions, len(opencc.DICTIONARIES))
                self.assertEqual(rules, active)
            finally:
                conn.close()

    def test_v01_database_is_migrated_instead_of_silently_rewritten(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "legacy.sqlite"
            opencc.import_many(
                inputs=self.opencc_inputs,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.old_schema,
                retrieved_at="2026-10-07T00:00:00Z",
                reset=True,
                fixture_mode=True,
            )
            conn = sqlite3.connect(db_path)
            try:
                columns = {
                    row[1] for row in conn.execute("PRAGMA table_info(source_versions)")
                }
                self.assertIn("resource_key", columns)
                self.assertIn("is_current", columns)
                version = conn.execute(
                    "SELECT value FROM build_metadata WHERE key = 'schema_version'"
                ).fetchone()[0]
                self.assertEqual(version, "0.2")
            finally:
                conn.close()

    def test_wikidata_refresh_supersedes_old_preferred_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            db_path = tmp_path / "core.sqlite"
            self._build_core(db_path)

            document = json.loads(
                (REPO_ROOT / "data" / "fixtures" / "wikidata" / "entities.json").read_text(
                    encoding="utf-8"
                )
            )
            pitt = document["entities"]["Q35332"]
            pitt["lastrevid"] = 1000003
            pitt["modified"] = "2026-10-07T00:00:00Z"
            pitt["labels"]["zh-hk"]["value"] = "畢彼特新名"
            refresh = tmp_path / "refresh.json"
            refresh.write_text(
                json.dumps({"entities": {"Q35332": pitt}}, ensure_ascii=False),
                encoding="utf-8",
            )
            type_map = tmp_path / "types.json"
            type_map.write_text(json.dumps({"Q35332": "person"}), encoding="utf-8")

            wikidata.import_document(
                input_path=refresh,
                type_map_path=type_map,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-07T01:00:00Z",
                reset=False,
                fixture_mode=True,
            )

            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                result = LocalizerEngine(conn).localize(
                    "布拉德·皮特", "zh-CN", "zh-HK"
                )
                self.assertEqual(result["output"], "畢彼特新名")
                self.assertFalse(result["review_needed"])
                versions = conn.execute(
                    """
                    SELECT is_current FROM source_versions
                    WHERE source_id = 'wikidata' AND resource_key = 'wikidata:Q35332'
                    ORDER BY source_version_id
                    """
                ).fetchall()
                self.assertEqual([row[0] for row in versions], [0, 1])
            finally:
                conn.close()

    def test_short_common_word_entity_is_not_auto_interpreted(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "core.sqlite"
            self._build_core(db_path)
            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                cursor = conn.execute(
                    "INSERT INTO concepts (concept_type, canonical_key, domain) VALUES ('product', 'test:apple', 'entity')"
                )
                concept_id = int(cursor.lastrowid)
                conn.execute(
                    """
                    INSERT INTO localized_names
                    (concept_id, locale, text, name_type, domain, is_preferred, confidence)
                    VALUES (?, 'zh-CN', '苹果', 'preferred', 'entity', 1, 1.0)
                    """,
                    (concept_id,),
                )
                conn.execute(
                    """
                    INSERT INTO localized_names
                    (concept_id, locale, text, name_type, domain, is_preferred, confidence)
                    VALUES (?, 'zh-HK', '蘋果公司', 'preferred', 'entity', 1, 1.0)
                    """,
                    (concept_id,),
                )
                conn.commit()
                result = LocalizerEngine(conn).localize(
                    "我吃苹果", "zh-CN", "zh-HK"
                )
                self.assertEqual(result["output"], "我吃苹果")
                self.assertFalse(any(event["type"] == "entity" for event in result["changes"]))
            finally:
                conn.close()

    def test_original_to_final_alignment_is_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "core.sqlite"
            self._build_core(db_path)
            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                result = LocalizerEngine(conn).localize(
                    "布拉德·皮特研究人工智能", "zh-CN", "zh-TW"
                )
                self.assertEqual(result["output"], "布萊德·彼特研究人工智慧")
                entity = next(e for e in result["changes"] if e["type"] == "entity")
                term = next(e for e in result["changes"] if e["type"] == "term_rule")
                self.assertEqual(entity["original_input_span"], [0, 6])
                self.assertEqual(entity["final_output_span"], [0, 6])
                self.assertEqual(term["original_input_span"], [8, 12])
                self.assertEqual(term["final_output_span"], [8, 12])
            finally:
                conn.close()

    def test_context_constraints_are_enforced(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "core.sqlite"
            self._build_core(db_path)
            conn = sqlite3.connect(db_path)
            conn.row_factory = sqlite3.Row
            try:
                version_id = conn.execute(
                    "SELECT source_version_id FROM source_versions WHERE source_id = 'opencc' AND is_current = 1 LIMIT 1"
                ).fetchone()[0]
                constraint = json.dumps({"constraints": {"domain": "technology"}})
                conn.execute(
                    """
                    INSERT INTO term_rules (
                        source_locale, target_locale, source_text, target_text,
                        domain, rule_type, priority, context_constraint,
                        source_id, source_version_id, upstream_record_id,
                        upstream_url, confidence, active
                    ) VALUES ('zh-Hant', 'zh-HK', '測試詞', '領域詞',
                              'test', 'test_rule', 999999, ?, 'opencc', ?,
                              'test:context', 'fixture://test', 1.0, 1)
                    """,
                    (constraint, version_id),
                )
                conn.commit()
                engine = LocalizerEngine(conn)
                self.assertEqual(
                    engine.localize("測試詞", "zh-Hant", "zh-HK")["output"],
                    "測試詞",
                )
                self.assertEqual(
                    engine.localize(
                        "測試詞",
                        "zh-Hant",
                        "zh-HK",
                        context={"domain": "technology"},
                    )["output"],
                    "領域詞",
                )
            finally:
                conn.close()

    def test_licence_pack_mixing_is_blocked(self):
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "core.sqlite"
            opencc.import_many(
                inputs=self.opencc_inputs,
                db_path=db_path,
                manifest_path=self.manifest,
                schema_path=self.schema,
                retrieved_at="2026-10-07T00:00:00Z",
                reset=True,
                fixture_mode=True,
            )
            with self.assertRaises(poc_builder.IngestPolicyError):
                lshk.import_tsv(
                    input_path=REPO_ROOT / "data" / "fixtures" / "lshk_sample.tsv",
                    db_path=db_path,
                    manifest_path=self.manifest,
                    schema_path=self.schema,
                    migration_path=REPO_ROOT
                    / "schema"
                    / "migrations"
                    / "0002_pronunciations.sql",
                    revision_id="test-fixture-revision",
                    upstream_url="fixture://lshk_sample.tsv",
                    retrieved_at="2026-10-07T00:00:00Z",
                    reset=False,
                )

    def test_pinned_lshk_revision_rejects_wrong_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            bad = tmp_path / "list.tsv"
            bad.write_text(
                "CH\tUCODE\tJP\tINIT\tFINL\tTONE\tDESC\tDESC_JP\n"
                "的\tU+7684\tdik1\td\tik\t1\t\t\n",
                encoding="utf-8",
            )
            with self.assertRaises(lshk.LshkFormatError):
                lshk.import_tsv(
                    input_path=bad,
                    db_path=tmp_path / "lshk.sqlite",
                    manifest_path=self.manifest,
                    schema_path=self.schema,
                    migration_path=REPO_ROOT
                    / "schema"
                    / "migrations"
                    / "0002_pronunciations.sql",
                    revision_id=lshk.PINNED_COMMIT,
                    upstream_url=lshk.PINNED_URL,
                    retrieved_at="2026-10-07T00:00:00Z",
                    reset=True,
                )


if __name__ == "__main__":
    unittest.main()
