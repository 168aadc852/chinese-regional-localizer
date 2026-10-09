"""Shared #48 golden fixtures and live Rust/Python candidate/edit parity support."""

import json
import sqlite3
import sys
import unittest
from copy import deepcopy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))
from alternative_terms import ChoiceError, ReviewSession  # noqa: E402
from context_profiles import ContextProfile, ContextProfileError, ContextProfiles  # noqa: E402
from localizer_engine import LocalizerEngine  # noqa: E402
from user_dictionary import UserDictionary  # noqa: E402
from user_localizer import UserControlledLocalizer  # noqa: E402

BASE = json.loads(
    (ROOT / "data/fixtures/context_selection_cases.json").read_text(encoding="utf-8")
)
FIXTURE = json.loads(
    (ROOT / "data/fixtures/alternative_terms_cases.json").read_text(encoding="utf-8")
)
RULES = BASE["rules"] | FIXTURE["rules"]
CASES = BASE["cases"] + FIXTURE["cases"]


def run_case(case, reverse=False):
    conn = sqlite3.connect(":memory:")
    user = None
    try:
        conn.executescript(
            (ROOT / "schema/sqlite-v0.2.sql").read_text(encoding="utf-8")
        )
        conn.execute(
            "INSERT INTO sources(source_id,name,status,pack,ingest_allowed,review_record,manifest_version) VALUES ('fixture','Synthetic fixture','approved','core',0,'project-authored test only',1)"
        )
        conn.execute(
            "INSERT INTO source_versions(source_version_id,source_id,is_current,retrieved_at) VALUES (1,'fixture',1,'2026-10-10'),(2,'fixture',0,'2026-10-10')"
        )
        ids = {key: i + 1 for i, key in enumerate(sorted(RULES))}
        keys = list(case["rules"])
        if reverse:
            keys.reverse()
        for key in keys:
            rule = RULES[key]
            constraint = (
                rule["raw_constraint"]
                if "raw_constraint" in rule
                else json.dumps(rule["constraint"])
                if "constraint" in rule
                else None
            )
            conn.execute(
                "INSERT INTO term_rules(rule_id,source_text,target_text,source_locale,target_locale,priority,context_constraint,rule_type,source_id,source_version_id,upstream_record_id,active) VALUES (?,?,?,?,?,?,?,'fixture','fixture',?,?,?)",
                (
                    ids[key],
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
        if case.get("entity"):
            for id in range(1, 3 if case.get("ambiguous_entity") else 2):
                conn.execute(
                    "INSERT INTO concepts(concept_id,concept_type,domain) VALUES (?,'person','entity')",
                    (id,),
                )
                for locale, text in [
                    ("zh-Hant", "測試人物"),
                    ("zh-HK", case.get("entity_target", "人物名稱")),
                ]:
                    conn.execute(
                        "INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (?,?,?,'preferred','entity',1,1)",
                        (id, locale, text),
                    )
            if case.get("ambiguous_target"):
                conn.execute(
                    "INSERT INTO localized_names(concept_id,locale,text,name_type,domain,is_preferred,confidence) VALUES (1,'zh-HK','另一名稱','preferred','entity',1,1)"
                )
        conn.commit()
        defaults = list(ContextProfiles()._profiles.values())
        engine = LocalizerEngine(
            conn,
            context_profiles=ContextProfiles(
                defaults + [ContextProfile(**item) for item in BASE["profiles"]]
            ),
        )
        if case.get("user_kind"):
            user = sqlite3.connect(":memory:")
            user.executescript(
                (ROOT / "schema/user-dictionary-v0.1.sql").read_text(encoding="utf-8")
            )
            user.execute(
                "INSERT INTO user_terms(kind,source_text,replacement,created_at,updated_at) VALUES (?,'測試詞',?,'2026-10-10','2026-10-10')",
                (
                    case["user_kind"],
                    case.get("override_target", "個人詞")
                    if case["user_kind"] == "override"
                    else None,
                ),
            )
            engine = UserControlledLocalizer(engine, UserDictionary(user))
        return engine.localize(
            case.get("input", "測試詞"),
            case.get("source_locale", "zh-Hant"),
            case.get("target_locale", "zh-HK"),
            context=case.get("context"),
        )
    finally:
        conn.close()
        if user is not None:
            user.close()


def choices(result):
    return [event["choice"] for event in result["changes"] if "choice" in event]


def edit_trace(case, result):
    session = ReviewSession(result)  # all DB connections are already closed
    trace = [session.snapshot()]
    for edit in case.get("edit_script", []):
        current = session.snapshot()
        if edit.get("undo"):
            trace.append(session.undo(current["revision"]))
        else:
            item = current["occurrences"][edit["occurrence"]]
            candidate = next(
                value
                for value in item["candidates"]
                if value["target_text"] == edit["target"]
            )
            trace.append(
                session.apply_choice(
                    current["revision"],
                    item["occurrence_id"],
                    candidate["candidate_id"],
                )
            )
    return trace


def observable(case, reverse=False):
    try:
        result = run_case(case, reverse)
    except ContextProfileError:
        return {"error": True}
    return {
        "output": result["output"],
        "review_needed": result["review_needed"],
        "choices": choices(result),
        "trace": edit_trace(case, result),
    }


class AlternativeTermsTests(unittest.TestCase):
    def test_golden_candidates_spans_and_repeatability(self):
        for case in CASES:
            with self.subTest(case=case["id"]):
                actual = observable(case)
                if case.get("error"):
                    self.assertEqual(actual, {"error": True})
                    continue
                self.assertEqual(actual["output"], case["expected"])
                self.assertEqual(actual["review_needed"], case.get("review", False))
                items = actual["choices"]
                if "expected_candidates" in case:
                    self.assertTrue(items)
                    for item in items:
                        self.assertEqual(
                            [
                                [c["target_text"], c["status"]]
                                for c in item["candidates"]
                            ],
                            case["expected_candidates"],
                        )
                if "expected_source_spans" in case:
                    self.assertEqual(
                        [item["source_span"] for item in items],
                        case["expected_source_spans"],
                    )
                if "expected_output_spans" in case:
                    self.assertEqual(
                        [item["output_span"] for item in items],
                        case["expected_output_spans"],
                    )
                for index, item in enumerate(items):
                    self.assertEqual(item["occurrence_id"], f"occurrence-{index + 1}")
                    a, b = item["output_span"]
                    self.assertEqual(actual["output"][a:b], item["expected_text"])
                    self.assertEqual(
                        len({c["target_text"] for c in item["candidates"]}),
                        len(item["candidates"]),
                    )
                    self.assertTrue(
                        all(
                            set(c) == {"candidate_id", "target_text", "status"}
                            for c in item["candidates"]
                        )
                    )
                self.assertEqual(observable(case, True), actual)
                self.assertEqual(observable(case), actual)

    def test_edit_and_undo_trace_matches_independent_splice_model(self):
        for case in FIXTURE["cases"]:
            if not case.get("edit_script"):
                continue
            with self.subTest(case=case["id"]):
                actual = observable(case)
                expected = deepcopy(actual["trace"][0])
                history = []
                for revision, (edit, snapshot) in enumerate(
                    zip(case["edit_script"], actual["trace"][1:]), 1
                ):
                    if edit.get("undo"):
                        expected = history.pop()
                    else:
                        history.append(deepcopy(expected))
                        item = expected["occurrences"][edit["occurrence"]]
                        start, end = item["output_span"]
                        before, after = expected["text"][:start], expected["text"][end:]
                        expected["text"] = before + edit["target"] + after
                        delta = len(edit["target"]) - (end - start)
                        item["output_span"] = [start, start + len(edit["target"])]
                        item["expected_text"] = edit["target"]
                        item["state"] = "chosen"
                        item["selected_candidate_id"] = next(
                            c["candidate_id"]
                            for c in item["candidates"]
                            if c["target_text"] == edit["target"]
                        )
                        for later in expected["occurrences"][edit["occurrence"] + 1 :]:
                            later["output_span"] = [
                                p + delta for p in later["output_span"]
                            ]
                    expected["revision"] = revision
                    self.assertEqual(snapshot, expected)
                self.assertEqual(actual["trace"][-1]["text"], actual["output"])
                self.assertEqual(actual["trace"][-1]["occurrences"], actual["choices"])

    def test_invalid_choices_are_atomic_and_snapshot_is_not_live(self):
        case = next(
            c for c in FIXTURE["cases"] if c["id"] == "unicode-adjacent-repeated"
        )
        session = ReviewSession(run_case(case))
        initial = session.snapshot()
        first, second = initial["occurrences"]
        operations = [
            lambda: session.apply_choice(
                99, first["occurrence_id"], first["candidates"][0]["candidate_id"]
            ),
            lambda: session.apply_choice(
                0, "unknown", first["candidates"][0]["candidate_id"]
            ),
            lambda: session.apply_choice(0, first["occurrence_id"], "unknown"),
            lambda: session.apply_choice(
                0, first["occurrence_id"], second["candidates"][0]["candidate_id"]
            ),
            lambda: session.apply_choice(
                0,
                first["occurrence_id"],
                first["candidates"][0]["candidate_id"],
                "remember_for_this_context",
            ),
            lambda: session.apply_choice(
                0,
                first["occurrence_id"],
                first["candidates"][0]["candidate_id"],
                "remember_for_all_contexts",
            ),
            lambda: session.undo(0),
        ]
        for operation in operations:
            with self.assertRaises(ChoiceError):
                operation()
            self.assertEqual(session.snapshot(), initial)
        modified = session.snapshot()
        modified["occurrences"][0]["candidates"][0]["target_text"] = "tampered"
        self.assertEqual(session.snapshot(), initial)
        session.apply_choice(
            0, first["occurrence_id"], first["candidates"][0]["candidate_id"]
        )
        changed = session.snapshot()
        with self.assertRaises(ChoiceError):
            session.undo(0)
        self.assertEqual(session.snapshot(), changed)

    def test_stale_anchor_overlap_and_lost_candidate_fail_closed(self):
        case = next(
            c for c in FIXTURE["cases"] if c["id"] == "unicode-adjacent-repeated"
        )
        for corruption in ("text", "overlap", "candidate", "bounds"):
            session = ReviewSession(run_case(case))
            first = session.snapshot()["occurrences"][0]
            if corruption == "text":
                session._text = "X" + session._text[1:2] + "錯" + session._text[3:]
            elif corruption == "overlap":
                session._occurrences[1]["output_span"] = [2, 5]
            elif corruption == "candidate":
                session._occurrences[0]["candidates"] = []
            else:
                session._occurrences[0]["output_span"] = [0, 99999]
            before = session.snapshot()
            with self.assertRaises(ChoiceError):
                session.apply_choice(
                    0, first["occurrence_id"], first["candidates"][0]["candidate_id"]
                )
            self.assertEqual(session.snapshot(), before)
            with self.assertRaises(ChoiceError):
                session.undo(0)
            self.assertEqual(session.snapshot(), before)

    def test_stale_undo_and_revision_exhaustion_are_atomic(self):
        case = next(
            c for c in FIXTURE["cases"] if c["id"] == "unicode-adjacent-repeated"
        )
        session = ReviewSession(run_case(case))
        item = session.snapshot()["occurrences"][0]
        session.apply_choice(
            0, item["occurrence_id"], item["candidates"][0]["candidate_id"]
        )
        session._text = session._text[:1] + "錯" + session._text[2:]
        before = session.snapshot()
        with self.assertRaises(ChoiceError):
            session.undo(1)
        self.assertEqual(session.snapshot(), before)
        self.assertEqual(len(session._undo), 1)
        exhausted = ReviewSession(run_case(case))
        exhausted._revision = (1 << 64) - 1
        before = exhausted.snapshot()
        with self.assertRaises(ChoiceError):
            exhausted.apply_choice(
                before["revision"],
                item["occurrence_id"],
                item["candidates"][0]["candidate_id"],
            )
        self.assertEqual(exhausted.snapshot(), before)


if __name__ == "__main__":
    if sys.argv[1:] == ["--emit-parity"]:
        print(
            json.dumps(
                {case["id"]: observable(case) for case in CASES},
                ensure_ascii=False,
                sort_keys=True,
            )
        )
    else:
        unittest.main()
