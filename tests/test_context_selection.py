"""Project-authored selection cases, also executed by Rust for live reference parity."""

import json
import sqlite3
import sys
import unittest
from dataclasses import FrozenInstanceError, replace
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))
from context_profiles import ContextProfile, ContextProfileError, ContextProfiles  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402
from user_dictionary import UserDictionary  # noqa: E402
from user_localizer import UserControlledLocalizer  # noqa: E402

FIXTURE = json.loads(
    (ROOT / "data/fixtures/context_selection_cases.json").read_text(encoding="utf-8")
)


def profiles():
    defaults = list(ContextProfiles()._profiles.values())
    return ContextProfiles(
        defaults + [ContextProfile(**item) for item in FIXTURE["profiles"]]
    )


def shared_database(
    rules, *, reverse=False, entity=False, ambiguous=False, ambiguous_target=False
):
    conn = sqlite3.connect(":memory:")
    conn.executescript((ROOT / "schema/sqlite-v0.2.sql").read_text(encoding="utf-8"))
    conn.execute(
        "INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic fixture','approved','core',0,'project-authored test only',1)"
    )
    conn.execute(
        "INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-09')"
    )
    conn.execute(
        "INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (2,'fixture',0,'2026-10-09')"
    )
    ids = {key: i + 1 for i, key in enumerate(sorted(FIXTURE["rules"]))}
    for key, rule in list(rules.items())[:: -1 if reverse else 1]:
        constraint = (
            rule.get("raw_constraint")
            if "raw_constraint" in rule
            else json.dumps(rule["constraint"])
            if "constraint" in rule
            else None
        )
        conn.execute(
            "INSERT INTO term_rules(rule_id,source_text,target_text,source_locale,target_locale,priority,context_constraint,rule_type,source_id,source_version_id,upstream_record_id,active) VALUES (?,?,?,?,?,?,?,'fixture','fixture',?,?,?)",
            (
                ids.get(key, 10000),
                rule["source"],
                rule["target"],
                rule.get("source_locale", "zh-Hant"),
                rule.get("target_locale", "zh-HK"),
                rule.get("priority", 0),
                constraint,
                2 if rule.get("stale") else 1,
                key,
                int(rule.get("active", True)),
            ),
        )
    if entity:
        for i in range(1, 3 if ambiguous else 2):
            conn.execute(
                "INSERT INTO concepts(concept_id,concept_type,domain) VALUES (?,'person','entity')",
                (i,),
            )
            conn.execute(
                "INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (?,'zh-Hant','測試人物','preferred','entity',1,1)",
                (i,),
            )
            conn.execute(
                "INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (?,'zh-HK','人物名稱','preferred','entity',1,1)",
                (i,),
            )
        if ambiguous_target:
            conn.execute(
                "INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (1,'zh-HK','另一名稱','preferred','entity',1,1)"
            )
    conn.commit()
    return conn


def run_case(case, *, reverse=False):
    rules = {key: FIXTURE["rules"][key] for key in case["rules"]}
    conn = shared_database(
        rules,
        reverse=reverse,
        entity=case.get("entity", False),
        ambiguous=case.get("ambiguous_entity", False),
        ambiguous_target=case.get("ambiguous_target", False),
    )
    user = None
    try:
        engine = LocalizerEngine(conn, context_profiles=profiles())
        if case.get("user_kind"):
            user = sqlite3.connect(":memory:")
            user.executescript(
                (ROOT / "schema/user-dictionary-v0.1.sql").read_text(encoding="utf-8")
            )
            user.execute(
                "INSERT INTO user_terms(kind,source_text,replacement,created_at,updated_at) VALUES (?,'測試詞',?,'2026-10-09','2026-10-09')",
                (
                    case["user_kind"],
                    "個人詞" if case["user_kind"] == "override" else None,
                ),
            )
            engine = UserControlledLocalizer(engine, UserDictionary(user))
        result = engine.localize(
            case.get("input", "測試詞"),
            case.get("source_locale", "zh-Hant"),
            case.get("target_locale", "zh-HK"),
            context=case.get("context"),
        )
        return result
    finally:
        if user is not None:
            user.close()
        conn.close()


def observable(case, *, reverse=False):
    try:
        result = run_case(case, reverse=reverse)
        return {
            "output": result["output"],
            "review_needed": result["review_needed"],
            "context_selections": [
                event["context_selection"]
                for event in result["changes"]
                if "context_selection" in event
            ],
        }
    except ContextProfileError:
        return {"error": True}


class ContextSelectionTests(unittest.TestCase):
    def test_shared_contract_cases_and_repeatability(self):
        for case in FIXTURE["cases"]:
            with self.subTest(case=case["id"]):
                result = observable(case)
                if case.get("error"):
                    self.assertEqual(result, {"error": True})
                else:
                    self.assertEqual(result["output"], case["expected"])
                    self.assertEqual(result["review_needed"], case.get("review", False))
                    if "level" in case:
                        expected = {
                            "usage_context_id": case["context"]["usage_context_id"],
                            "matched_usage_context_id": case["matched"],
                            "context_distance": case["distance"],
                            "context_level": case["level"],
                        }
                        self.assertTrue(result["context_selections"])
                        self.assertTrue(
                            all(
                                value == expected
                                for value in result["context_selections"]
                            )
                        )
                for _ in range(2):
                    self.assertEqual(observable(case), result)
                    self.assertEqual(observable(case, reverse=True), result)

    def test_malformed_rule_conditions_fail_closed(self):
        bad = [
            None,
            [],
            "technology-software",
            5,
            {"constraints": None},
            {"usage_context_id": "technology-software"},
        ]
        bad += [
            {"constraints": {"usage_context_id": value}}
            for value in [
                None,
                [],
                {},
                True,
                3,
                "",
                "*",
                "unknown",
                ["technology-software"],
            ]
        ]
        for metadata in bad:
            with self.subTest(metadata=metadata):
                conn = shared_database(
                    {
                        "bad": {
                            "source": "測試詞",
                            "target": "不應使用",
                            "constraint": metadata,
                        }
                    }
                )
                try:
                    result = LocalizerEngine(conn).localize(
                        "測試詞",
                        "zh-Hant",
                        "zh-HK",
                        context={"usage_context_id": "technology-software"},
                    )
                    self.assertEqual(result["output"], "測試詞")
                finally:
                    conn.close()
        for raw in ["{", "", '{"constraints":{"usage_context_id":null}}']:
            conn = shared_database(
                {
                    "bad": {
                        "source": "測試詞",
                        "target": "不應使用",
                        "raw_constraint": raw,
                    }
                }
            )
            try:
                self.assertEqual(
                    LocalizerEngine(conn).localize(
                        "測試詞",
                        "zh-Hant",
                        "zh-HK",
                        context={"usage_context_id": "technology-software"},
                    )["output"],
                    "測試詞",
                )
            finally:
                conn.close()

    def test_profile_adapter_preserves_46_validation_and_immutable_snapshot(self):
        snapshot = profiles()
        with self.assertRaises(FrozenInstanceError):
            snapshot._profiles = {}
        defaults = list(ContextProfiles()._profiles.values())
        for custom in [
            [ContextProfile("general", "Replacement")],
            [ContextProfile("a", "A", "missing")],
            [ContextProfile("a", "A", "a")],
            [ContextProfile("a", "A", "b"), ContextProfile("b", "B", "a")],
            [ContextProfile("a", "A"), ContextProfile("a", "Duplicate")],
        ]:
            with self.assertRaises(ContextProfileError):
                ContextProfiles(defaults + custom)
        document = {
            "schema_version": 1,
            "profiles": [vars(item) for item in snapshot._profiles.values()],
        }
        self.assertEqual(
            ContextProfiles.from_json(json.dumps(document)).resolve_chain(
                "hi-fi-audio"
            ),
            snapshot.resolve_chain("hi-fi-audio"),
        )
        document["schema_version"] = 2
        with self.assertRaises(ContextProfileError):
            ContextProfiles.from_json(json.dumps(document))
        disabled = ContextProfiles(
            [
                replace(item, enabled=False) if item.context_id == "general" else item
                for item in defaults
            ]
        )
        with self.assertRaises(ContextProfileError):
            disabled.resolve_chain("legal")

    def test_selected_rule_provenance_and_legacy_events(self):
        case = next(
            case
            for case in FIXTURE["cases"]
            if case["id"] == "actual-winner-provenance"
        )
        event = next(
            event for event in run_case(case)["changes"] if "context_selection" in event
        )
        self.assertEqual(event["upstream_record_id"], "technology")
        legacy = observable(FIXTURE["cases"][0])
        self.assertEqual(legacy["context_selections"], [])


if __name__ == "__main__":
    if sys.argv[1:] == ["--emit-parity"]:
        print(
            json.dumps(
                {case["id"]: observable(case) for case in FIXTURE["cases"]},
                ensure_ascii=False,
                sort_keys=True,
            )
        )
    else:
        unittest.main()
