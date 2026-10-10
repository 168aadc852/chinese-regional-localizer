"""#49 synthetic private selection, migration, operations and remembered-write parity."""

import json
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))
from alternative_terms import ChoiceError  # noqa: E402
from context_profiles import ContextProfiles, ContextProfile  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402
from private_store import PrivateStore, prepare_store  # noqa: E402
from private_review import PersistenceError  # noqa: E402
from user_dictionary import UserDictionary  # noqa: E402
from user_localizer import UserControlledLocalizer  # noqa: E402

CASES = json.loads(
    (ROOT / "data/fixtures/private_preferences_cases.json").read_text(encoding="utf-8")
)["cases"]
BASE = json.loads(
    (ROOT / "data/fixtures/context_selection_cases.json").read_text(encoding="utf-8")
)


def profiles():
    return ContextProfiles(
        list(ContextProfiles()._profiles.values())
        + [ContextProfile(**item) for item in BASE["profiles"]]
    )


def shared():
    conn = sqlite3.connect(":memory:")
    conn.executescript((ROOT / "schema/sqlite-v0.2.sql").read_text(encoding="utf-8"))
    conn.execute(
        "INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic','approved','core',0,'project-authored',1)"
    )
    conn.execute(
        "INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-10')"
    )
    for source, target, a, b, priority in [
        ("測試詞", "官方詞", "zh-Hant", "zh-HK", 10),
        ("測試詞", "另詞", "zh-Hant", "zh-HK", 1),
        ("測試詞", "台灣詞", "zh-Hant", "zh-TW", 10),
        ("测试词", "測試詞", "zh-CN", "zh-Hant", 0),
    ]:
        conn.execute(
            "INSERT INTO term_rules(source_text,target_text,source_locale,target_locale,priority,rule_type,source_id,source_version_id) VALUES (?,?,?,?,?,'fixture','fixture',1)",
            (source, target, a, b, priority),
        )
    conn.commit()
    return conn


def legacy(conn):
    conn.executescript(
        (ROOT / "schema/user-dictionary-v0.1.sql").read_text(encoding="utf-8")
    )
    conn.execute(
        "INSERT INTO user_terms(user_term_id,kind,source_text,replacement,source_locale,target_locale,priority,enabled,note,created_at,updated_at) VALUES (42,'protected','OpenAI',NULL,NULL,NULL,77,1,'保留','old','old'),(99,'override','測試詞','舊詞','zh-Hant',NULL,123,0,'note','old','old')"
    )
    conn.commit()


def select_case(case, reverse=False):
    conn = shared()
    user = sqlite3.connect(":memory:")
    store = PrivateStore(user, profiles())
    dictionary = UserDictionary(user)
    terms = list(case["terms"])
    if reverse:
        terms.reverse()
    ids = sorted({term[0] for term in terms} - {"personal", "legacy"}, reverse=reverse)
    for id in ids:
        store.create_dictionary(id, id)
    for term in terms:
        id = store.put_preference(
            term[0],
            term[1],
            term[2],
            term[4] if len(term) > 4 else "zh-HK",
            term[3],
            term[5] if len(term) > 5 else None,
        )
        if len(term) > 6 and not term[6]:
            store.set_term_enabled(id, False)
    for id in case.get("disabled", []):
        store.set_dictionary_enabled(id, False)
    for source, target, a, b, priority in case.get("legacy", []):
        dictionary.add_override(
            source, target, source_locale=a, target_locale=b, priority=priority
        )
    if case.get("protect"):
        dictionary.add_protected("測試詞")
    engine = UserControlledLocalizer(
        LocalizerEngine(conn, context_profiles=profiles()), dictionary
    )
    context = case.get("context", "technology-software")
    result = engine.localize(
        case.get("input", "測試詞"),
        case.get("source", "zh-Hant"),
        case.get("target", "zh-HK"),
        context={"usage_context_id": context} if context else None,
    )
    from alternative_terms import ReviewSession

    snapshot = ReviewSession(result).snapshot()
    answer = {
        "output": result["output"],
        "review": result["review_needed"],
        "snapshot": snapshot,
    }
    conn.close()
    store.close()
    return answer


def persistence_trace(intent, fail=False):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "user.sqlite"
        store = PrivateStore.open(path, profiles())
        conn = shared()
        engine = UserControlledLocalizer(
            LocalizerEngine(conn, context_profiles=profiles()),
            UserDictionary(store.conn),
        )
        session = engine.review(
            "😀测试词测试词",
            "zh-CN",
            "zh-HK",
            context={"usage_context_id": "technology-software"},
        )
        item = session.snapshot()["occurrences"][0]
        candidate = next(c for c in item["candidates"] if c["target_text"] == "另詞")
        if fail:
            store.conn.execute(
                "CREATE TRIGGER fail_write BEFORE INSERT ON user_terms BEGIN SELECT RAISE(ABORT,'injected failure'); END"
            )
        error = False
        try:
            session.apply_choice(
                0, item["occurrence_id"], candidate["candidate_id"], intent, store
            )
        except PersistenceError:
            error = True
        snapshot = session.snapshot()
        undo = session.undo(snapshot["revision"])
        store.close()
        reopened = PrivateStore.open(path, profiles())
        rows = [
            list(row)
            for row in reopened.conn.execute(
                "SELECT dictionary_id,source_text,replacement,source_locale,target_locale,usage_context_id,legacy FROM user_terms ORDER BY user_term_id"
            )
        ]
        output = UserControlledLocalizer(
            LocalizerEngine(conn, context_profiles=profiles()),
            UserDictionary(reopened.conn),
        ).localize(
            "测试词",
            "zh-CN",
            "zh-HK",
            context={"usage_context_id": "technology-software"},
        )["output"]
        reopened.close()
        conn.close()
        return {
            "snapshot": snapshot,
            "undo": undo,
            "rows": rows,
            "error": error,
            "reopened_output": output,
        }


def csv_trace():
    inputs = [
        ("source_text,replacement\n詞,甲\n詞,乙\n", "zh-HK"),
        ("source_text,replacement\n詞,答案\n", "zh-HK"),
        (
            'source_text,replacement,target_locale,note\r\n"詞,一","替代\n文",zh-HK,"引號""備註"\r\n',
            None,
        ),
        ("source_text,replacement\n有效,答案\n,錯誤\n", "zh-HK"),
        ("source_text,replacement\n詞,答案\n", None),
        (
            "source_text,replacement,target_locale,usage_context_id\n詞,答案,zh-HK,disabled\n",
            None,
        ),
        ('source_text,replacement\n有效,答案\n"壞詞,答案', "zh-HK"),
        ("source_text,source_text,replacement\n詞,詞,答案\n", "zh-HK"),
        ("source_text,replacement\n詞\n", "zh-HK"),
    ]
    store = PrivateStore(sqlite3.connect(":memory:"), profiles())
    traces = []
    for text, target in inputs:
        preview = store.preview_csv(text, target)
        traces.append(
            {
                "rows": [
                    [
                        row["source"],
                        row["replacement"],
                        row["target"],
                        row["usage"],
                        row["note"],
                    ]
                    for row in preview["rows"]
                ],
                "errors": [error["row"] for error in preview["errors"]],
            }
        )
    store.close()
    return traces


class PrivatePreferencesTests(unittest.TestCase):
    def test_strict_unversioned_migration_and_unknown_extensions_preserved(self):
        conn = sqlite3.connect(":memory:")
        legacy(conn)
        conn.execute("DROP TABLE user_metadata")
        before = conn.execute(
            "SELECT * FROM user_terms ORDER BY user_term_id"
        ).fetchall()
        prepare_store(conn)
        self.assertEqual(
            [
                row[:11]
                for row in conn.execute(
                    "SELECT * FROM user_terms ORDER BY user_term_id"
                )
            ],
            before,
        )
        conn.close()
        for extension in (
            "CREATE TABLE extra(value TEXT)",
            "CREATE VIEW extra AS SELECT * FROM user_terms",
        ):
            conn = sqlite3.connect(":memory:")
            legacy(conn)
            conn.execute(extension)
            with self.assertRaises(ValueError):
                prepare_store(conn)
            self.assertEqual(
                conn.execute(
                    "SELECT value FROM user_metadata WHERE key='schema_version'"
                ).fetchone()[0],
                "0.1",
            )
            conn.close()

    def test_selection_golden_and_both_insertion_orders(self):
        for case in CASES:
            with self.subTest(case=case["id"]):
                result = select_case(case)
                self.assertEqual(result["output"], case["expected"])
                self.assertEqual(result["review"], case.get("review", False))
                self.assertEqual(result, select_case(case, True))

    def test_remember_intents_reopen_original_cn_source_and_locale(self):
        for intent, scope in [
            ("use_this_time_only", None),
            ("remember_for_this_context", "technology-software"),
            ("remember_for_all_contexts", None),
        ]:
            trace = persistence_trace(intent)
            self.assertFalse(trace["error"])
            self.assertEqual(trace["snapshot"]["text"], "😀另詞官方詞")
            self.assertEqual(trace["undo"]["text"], "😀官方詞官方詞")
            if intent == "use_this_time_only":
                self.assertEqual(trace["rows"], [])
                self.assertEqual(trace["reopened_output"], "官方詞")
            else:
                self.assertEqual(
                    trace["rows"],
                    [["personal", "测试词", "另詞", "zh-CN", "zh-HK", scope, 0]],
                )
                self.assertEqual(trace["reopened_output"], "另詞")

    def test_failed_write_preserves_valid_review_undo_and_db(self):
        trace = persistence_trace("remember_for_this_context", True)
        self.assertTrue(trace["error"])
        self.assertEqual(trace["rows"], [])
        self.assertEqual(trace["snapshot"]["revision"], 1)
        self.assertEqual(trace["undo"]["text"], "😀官方詞官方詞")

    def test_idempotent_personal_update_and_other_dictionary_untouched(self):
        conn = sqlite3.connect(":memory:")
        store = PrivateStore(conn)
        store.create_dictionary("company", "Company")
        company = store.put_preference("company", "詞", "公司詞", "zh-HK")
        first = store.put_preference("personal", "詞", "一", "zh-HK")
        self.assertEqual(first, store.put_preference("personal", "詞", "一", "zh-HK"))
        self.assertEqual(first, store.put_preference("personal", "詞", "二", "zh-HK"))
        self.assertEqual(
            conn.execute(
                "SELECT replacement FROM user_terms WHERE user_term_id=?", (company,)
            ).fetchone()[0],
            "公司詞",
        )
        self.assertEqual(
            conn.execute("SELECT count(*) FROM user_terms").fetchone()[0], 2
        )
        store.close()

    def test_stale_foreign_candidate_no_write_and_missing_context(self):
        conn = shared()
        store = PrivateStore(sqlite3.connect(":memory:"))
        engine = UserControlledLocalizer(
            LocalizerEngine(conn), UserDictionary(store.conn)
        )
        session = engine.review(
            "測試詞測試詞", "zh-Hant", "zh-HK", context={"usage_context_id": "general"}
        )
        first, second = session.snapshot()["occurrences"]
        for revision, candidate in [
            (1, first["candidates"][0]["candidate_id"]),
            (0, second["candidates"][0]["candidate_id"]),
            (0, "foreign"),
        ]:
            before = session.snapshot()
            with self.assertRaises(ChoiceError):
                session.apply_choice(
                    revision,
                    first["occurrence_id"],
                    candidate,
                    "remember_for_all_contexts",
                    store,
                )
            self.assertEqual(session.snapshot(), before)
        missing = engine.review("測試詞", "zh-Hant", "zh-HK")
        with self.assertRaises(ChoiceError):
            missing.apply_choice(
                0,
                "occurrence-1",
                "occurrence-1/candidate-1",
                "remember_for_this_context",
                store,
            )
        self.assertEqual(
            store.conn.execute("SELECT count(*) FROM user_terms").fetchone()[0], 0
        )
        conn.close()
        store.close()

    def test_migration_preserves_every_legacy_field_and_is_idempotent(self):
        conn = sqlite3.connect(":memory:")
        legacy(conn)
        before = conn.execute(
            "SELECT * FROM user_terms ORDER BY user_term_id"
        ).fetchall()
        prepare_store(conn)
        after = conn.execute(
            "SELECT * FROM user_terms ORDER BY user_term_id"
        ).fetchall()
        self.assertEqual([row[:11] for row in after], before)
        self.assertEqual(
            [row[11:] for row in after], [("legacy", None, 1), ("legacy", None, 1)]
        )
        prepare_store(conn)
        self.assertEqual(
            conn.execute("SELECT * FROM user_terms ORDER BY user_term_id").fetchall(),
            after,
        )
        conn.close()

    def test_injected_migration_failure_rolls_back_original_usable_schema(self):
        conn = sqlite3.connect(":memory:")
        legacy(conn)
        before = conn.execute(
            "SELECT * FROM user_terms ORDER BY user_term_id"
        ).fetchall()
        conn.execute(
            "CREATE TRIGGER fail_migration BEFORE UPDATE ON user_metadata BEGIN SELECT RAISE(ABORT,'injected'); END"
        )
        with self.assertRaises(sqlite3.DatabaseError):
            prepare_store(conn)
        self.assertEqual(
            conn.execute("SELECT * FROM user_terms ORDER BY user_term_id").fetchall(),
            before,
        )
        self.assertEqual(
            conn.execute(
                "SELECT value FROM user_metadata WHERE key='schema_version'"
            ).fetchone()[0],
            "0.1",
        )
        self.assertEqual(len(UserDictionary(conn).candidates("zh-Hant", "zh-HK")), 1)
        conn.execute("DROP TRIGGER fail_migration")
        prepare_store(conn)
        conn.close()

    def test_future_and_corrupt_fail_without_rewriting(self):
        conn = sqlite3.connect(":memory:")
        legacy(conn)
        conn.execute("UPDATE user_metadata SET value='999'")
        conn.commit()
        with self.assertRaises(ValueError):
            prepare_store(conn)
        self.assertEqual(
            conn.execute("SELECT value FROM user_metadata").fetchone()[0], "999"
        )
        conn.close()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "corrupt.sqlite"
            # A project-authored non-SQLite payload; no real user store is touched.
            path.write_bytes(b"not SQLite")
            with self.assertRaises(sqlite3.DatabaseError):
                PrivateStore.open(path)
            self.assertEqual(path.read_bytes(), b"not SQLite")

    def test_dictionary_operations_and_csv_revalidation_atomicity(self):
        store = PrivateStore(sqlite3.connect(":memory:"))
        store.create_dictionary("a", "Audio")
        store.rename_dictionary("a", "Hi-Fi Audio")
        for id in ("personal", "legacy"):
            with self.assertRaises(ValueError):
                store.rename_dictionary(id, "Wrong")
            with self.assertRaises(ValueError):
                store.delete_dictionary(id)
        preview = store.preview_csv(
            'source_text,replacement,target_locale,note\r\n"詞,一","替代\n文",zh-HK,"引號""備註"\r\n'
        )
        self.assertFalse(preview["errors"])
        store.commit_csv("a", preview)
        invalid = store.preview_csv(
            "source_text,replacement\n有效,答案\n,錯誤\n", "zh-HK"
        )
        self.assertEqual(invalid["errors"][0]["row"], 3)
        with self.assertRaises(ValueError):
            store.commit_csv("a", invalid)
        preview["rows"].append(
            {
                "source": "bad",
                "replacement": "X",
                "target": None,
                "usage": None,
                "note": None,
            }
        )
        with self.assertRaises(ValueError):
            store.commit_csv("a", preview)
        self.assertEqual(
            store.conn.execute("SELECT count(*) FROM user_terms").fetchone()[0], 1
        )
        id = store.conn.execute("SELECT user_term_id FROM user_terms").fetchone()[0]
        store.set_term_enabled(id, False)
        store.edit_preference(id, "新詞", "新答案", "zh-TW", "general")
        self.assertEqual(
            store.conn.execute(
                "SELECT source_text,target_locale,usage_context_id FROM user_terms WHERE user_term_id=?",
                (id,),
            ).fetchone(),
            ("新詞", "zh-TW", "general"),
        )
        store.delete_term(id)
        store.delete_dictionary("a")
        self.assertEqual(
            store.conn.execute("SELECT count(*) FROM user_terms").fetchone()[0], 0
        )
        with self.assertRaises(ValueError):
            store.put_preference("personal", "詞", "詞", None)
        store.close()

    def test_partial_script_expansion_remains_one_time_only(self):
        conn = shared()
        conn.execute(
            "INSERT INTO term_rules(source_text,target_text,source_locale,target_locale,priority,rule_type,source_id,source_version_id) VALUES ('甲','測試詞測試詞','zh-CN','zh-Hant',0,'fixture','fixture',1)"
        )
        conn.commit()
        store = PrivateStore(sqlite3.connect(":memory:"))
        engine = UserControlledLocalizer(
            LocalizerEngine(conn), UserDictionary(store.conn)
        )
        session = engine.review(
            "甲", "zh-CN", "zh-HK", context={"usage_context_id": "general"}
        )
        first = session.snapshot()["occurrences"][0]
        self.assertFalse(first["rememberable"])
        before = session.snapshot()
        with self.assertRaises(ChoiceError):
            session.apply_choice(
                0,
                first["occurrence_id"],
                first["candidates"][1]["candidate_id"],
                "remember_for_all_contexts",
                store,
            )
        self.assertEqual(session.snapshot(), before)
        self.assertEqual(
            session.apply_choice(
                0, first["occurrence_id"], first["candidates"][1]["candidate_id"]
            )["text"],
            "另詞官方詞",
        )
        self.assertEqual(
            store.conn.execute("SELECT count(*) FROM user_terms").fetchone()[0], 0
        )
        conn.close()
        store.close()


if __name__ == "__main__":
    if "--emit-parity" in sys.argv:
        print(
            json.dumps(
                {
                    "cases": {case["id"]: select_case(case) for case in CASES},
                    "csv": csv_trace(),
                    "persistence": {
                        intent: persistence_trace(intent)
                        for intent in (
                            "use_this_time_only",
                            "remember_for_this_context",
                            "remember_for_all_contexts",
                        )
                    },
                    "failed_write": persistence_trace(
                        "remember_for_this_context", True
                    ),
                },
                ensure_ascii=False,
            )
        )
    else:
        unittest.main()
