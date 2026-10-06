"""Deterministic reference localization engine for Phase 2A.

This Python module defines behavior before the production Rust runtime is
implemented. It prioritizes known entities, protects entity replacements, and
then applies staged term rules with deterministic longest-match scanning.
"""

from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from typing import Any


ROUTES: dict[tuple[str, str], list[tuple[str, str]]] = {
    ("zh-CN", "zh-HK"): [("zh-CN", "zh-Hant"), ("zh-Hant", "zh-HK")],
    ("zh-CN", "zh-TW"): [("zh-CN", "zh-Hant"), ("zh-Hant", "zh-TW")],
    ("zh-Hant", "zh-HK"): [("zh-Hant", "zh-HK")],
    ("zh-Hant", "zh-TW"): [("zh-Hant", "zh-TW")],
}


class UnsupportedRouteError(ValueError):
    pass


@dataclass(frozen=True)
class ProtectedSpan:
    start: int
    end: int


class LocalizerEngine:
    def __init__(self, conn: sqlite3.Connection):
        self.conn = conn
        self.conn.row_factory = sqlite3.Row

    def localize(self, text: str, source_locale: str, target_locale: str) -> dict[str, Any]:
        route = ROUTES.get((source_locale, target_locale))
        if route is None:
            raise UnsupportedRouteError(f"Unsupported route: {source_locale} -> {target_locale}")

        entity_result = self._apply_entities(text, source_locale, target_locale)
        current_text = entity_result["text"]
        protected = entity_result["protected"]
        events = list(entity_result["events"])

        for stage_source, stage_target in route:
            stage = self._apply_term_stage(
                current_text,
                protected,
                stage_source,
                stage_target,
            )
            current_text = stage["text"]
            protected = stage["protected"]
            events.extend(stage["events"])

        return {
            "input": text,
            "output": current_text,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "route": [f"{a}->{b}" for a, b in route],
            "changes": events,
            "review_needed": any(event.get("review_needed", False) for event in events),
        }

    def _entity_index(self, source_locale: str) -> dict[str, list[dict[str, Any]]]:
        rows = self.conn.execute(
            """
            SELECT ln.localized_name_id, ln.concept_id, ln.text, ln.name_type,
                   ln.confidence, c.concept_type,
                   (SELECT external_value FROM external_ids e
                    WHERE e.concept_id = c.concept_id AND e.namespace = 'wikidata'
                    LIMIT 1) AS qid
            FROM localized_names ln
            JOIN concepts c ON c.concept_id = ln.concept_id
            WHERE ln.locale = ? AND ln.domain = 'entity'
            """,
            (source_locale,),
        ).fetchall()
        index: dict[str, dict[int, dict[str, Any]]] = {}
        for row in rows:
            bucket = index.setdefault(row["text"], {})
            concept_id = int(row["concept_id"])
            current = bucket.get(concept_id)
            candidate = dict(row)
            if current is None or (candidate.get("confidence") or 0) > (current.get("confidence") or 0):
                bucket[concept_id] = candidate
        return {text: list(by_concept.values()) for text, by_concept in index.items()}

    @staticmethod
    def _text_match_index(values: list[str]) -> dict[str, list[str]]:
        result: dict[str, list[str]] = {}
        for value in set(values):
            if value:
                result.setdefault(value[0], []).append(value)
        for items in result.values():
            items.sort(key=lambda value: (-len(value), value))
        return result

    def _target_entity_names(self, concept_id: int, target_locale: str) -> list[sqlite3.Row]:
        return self.conn.execute(
            """
            SELECT localized_name_id, text, confidence
            FROM localized_names
            WHERE concept_id = ? AND locale = ?
              AND name_type = 'preferred' AND is_preferred = 1
            ORDER BY confidence DESC, text
            """,
            (concept_id, target_locale),
        ).fetchall()

    def _name_evidence(self, localized_name_id: int) -> list[dict[str, Any]]:
        rows = self.conn.execute(
            """
            SELECT ne.source_id, ne.upstream_record_id, ne.upstream_url,
                   ne.upstream_revision, ne.evidence_type, ne.confidence,
                   ne.retrieved_at, sv.version_label, sv.revision_id,
                   sv.checksum_sha256
            FROM name_evidence ne
            LEFT JOIN source_versions sv ON sv.source_version_id = ne.source_version_id
            WHERE ne.localized_name_id = ?
            ORDER BY ne.evidence_id
            """,
            (localized_name_id,),
        ).fetchall()
        return [dict(row) for row in rows]

    def _apply_entities(self, text: str, source_locale: str, target_locale: str) -> dict[str, Any]:
        entity_index = self._entity_index(source_locale)
        match_index = self._text_match_index(list(entity_index))
        out: list[str] = []
        protected: list[ProtectedSpan] = []
        events: list[dict[str, Any]] = []
        i = 0
        out_len = 0

        while i < len(text):
            candidates = [
                key for key in match_index.get(text[i], []) if text.startswith(key, i)
            ]
            if not candidates:
                out.append(text[i])
                out_len += 1
                i += 1
                continue

            source_text = candidates[0]
            concepts = entity_index[source_text]
            input_start = i
            input_end = i + len(source_text)
            output_start = out_len

            if len(concepts) != 1:
                replacement = source_text
                out.append(replacement)
                out_len += len(replacement)
                protected.append(ProtectedSpan(output_start, out_len))
                events.append(
                    {
                        "type": "entity",
                        "applied": False,
                        "review_needed": True,
                        "reason": "ambiguous_source_entity",
                        "original": source_text,
                        "replacement": source_text,
                        "source_locale": source_locale,
                        "target_locale": target_locale,
                        "input_span": [input_start, input_end],
                        "output_span": [output_start, out_len],
                        "candidates": [
                            {
                                "concept_id": item["concept_id"],
                                "concept_type": item["concept_type"],
                                "qid": item["qid"],
                            }
                            for item in concepts
                        ],
                    }
                )
                i = input_end
                continue

            concept = concepts[0]
            target_names = self._target_entity_names(int(concept["concept_id"]), target_locale)
            distinct_targets = {row["text"] for row in target_names}
            if len(distinct_targets) != 1:
                replacement = source_text
                out.append(replacement)
                out_len += len(replacement)
                protected.append(ProtectedSpan(output_start, out_len))
                events.append(
                    {
                        "type": "entity",
                        "applied": False,
                        "review_needed": True,
                        "reason": "missing_target_name" if not target_names else "ambiguous_target_name",
                        "original": source_text,
                        "replacement": source_text,
                        "source_locale": source_locale,
                        "target_locale": target_locale,
                        "input_span": [input_start, input_end],
                        "output_span": [output_start, out_len],
                        "concept_id": concept["concept_id"],
                        "concept_type": concept["concept_type"],
                        "qid": concept["qid"],
                        "target_candidates": sorted(distinct_targets),
                    }
                )
                i = input_end
                continue

            target_row = target_names[0]
            replacement = target_row["text"]
            out.append(replacement)
            out_len += len(replacement)
            protected.append(ProtectedSpan(output_start, out_len))
            events.append(
                {
                    "type": "entity",
                    "applied": replacement != source_text,
                    "review_needed": False,
                    "reason": "localized_entity_name",
                    "original": source_text,
                    "replacement": replacement,
                    "source_locale": source_locale,
                    "target_locale": target_locale,
                    "input_span": [input_start, input_end],
                    "output_span": [output_start, out_len],
                    "concept_id": concept["concept_id"],
                    "concept_type": concept["concept_type"],
                    "qid": concept["qid"],
                    "confidence": target_row["confidence"],
                    "evidence": self._name_evidence(int(target_row["localized_name_id"])),
                }
            )
            i = input_end

        return {"text": "".join(out), "protected": protected, "events": events}

    def _term_rule_index(self, source_locale: str, target_locale: str) -> dict[str, list[dict[str, Any]]]:
        rows = self.conn.execute(
            """
            SELECT tr.rule_id, tr.source_text, tr.target_text, tr.priority,
                   tr.context_constraint, tr.source_id, tr.source_version_id,
                   tr.upstream_record_id, tr.upstream_url, tr.confidence,
                   tr.rule_type, sv.version_label, sv.revision_id,
                   sv.checksum_sha256
            FROM term_rules tr
            LEFT JOIN source_versions sv ON sv.source_version_id = tr.source_version_id
            WHERE tr.source_locale = ? AND tr.target_locale = ? AND tr.active = 1
            ORDER BY tr.source_text, tr.priority DESC, tr.rule_id
            """,
            (source_locale, target_locale),
        ).fetchall()
        index: dict[str, list[dict[str, Any]]] = {}
        for row in rows:
            item = dict(row)
            if item.get("context_constraint"):
                try:
                    item["context"] = json.loads(item["context_constraint"])
                except json.JSONDecodeError:
                    item["context"] = {"raw": item["context_constraint"]}
            else:
                item["context"] = None
            index.setdefault(row["source_text"], []).append(item)
        return index

    def _apply_term_stage(
        self,
        text: str,
        protected: list[ProtectedSpan],
        source_locale: str,
        target_locale: str,
    ) -> dict[str, Any]:
        rule_index = self._term_rule_index(source_locale, target_locale)
        match_index = self._text_match_index(list(rule_index))
        protected = sorted(protected, key=lambda span: span.start)
        span_cursor = 0
        out: list[str] = []
        new_protected: list[ProtectedSpan] = []
        events: list[dict[str, Any]] = []
        out_len = 0
        i = 0
        stage_name = f"{source_locale}->{target_locale}"

        while i < len(text):
            if span_cursor < len(protected) and i == protected[span_cursor].start:
                span = protected[span_cursor]
                chunk = text[span.start:span.end]
                start = out_len
                out.append(chunk)
                out_len += len(chunk)
                new_protected.append(ProtectedSpan(start, out_len))
                i = span.end
                span_cursor += 1
                continue

            next_protected_start = (
                protected[span_cursor].start if span_cursor < len(protected) else len(text)
            )
            keys = [
                key
                for key in match_index.get(text[i], [])
                if i + len(key) <= next_protected_start and text.startswith(key, i)
            ]
            if not keys:
                out.append(text[i])
                out_len += 1
                i += 1
                continue

            source_text = keys[0]
            rows = rule_index[source_text]
            max_priority = max(int(row["priority"]) for row in rows)
            winners = [row for row in rows if int(row["priority"]) == max_priority]
            winner_targets = {row["target_text"] for row in winners}
            input_start = i
            input_end = i + len(source_text)
            output_start = out_len

            if len(winner_targets) != 1:
                out.append(source_text)
                out_len += len(source_text)
                new_protected.append(ProtectedSpan(output_start, out_len))
                events.append(
                    {
                        "type": "term_rule",
                        "applied": False,
                        "review_needed": True,
                        "reason": "ambiguous_rule_tie",
                        "stage": stage_name,
                        "original": source_text,
                        "replacement": source_text,
                        "source_locale": source_locale,
                        "target_locale": target_locale,
                        "stage_input_span": [input_start, input_end],
                        "stage_output_span": [output_start, out_len],
                        "candidates": [self._public_rule(row) for row in winners],
                    }
                )
                i = input_end
                continue

            winner = winners[0]
            replacement = winner["target_text"]
            out.append(replacement)
            out_len += len(replacement)

            alternatives = []
            seen = {replacement}
            for row in rows:
                if row["target_text"] not in seen:
                    seen.add(row["target_text"])
                    alternatives.append(self._public_rule(row))

            if replacement != source_text:
                event = self._public_rule(winner)
                event.update(
                    {
                        "type": "term_rule",
                        "applied": True,
                        "review_needed": False,
                        "reason": "highest_priority_longest_match",
                        "stage": stage_name,
                        "original": source_text,
                        "replacement": replacement,
                        "source_locale": source_locale,
                        "target_locale": target_locale,
                        "stage_input_span": [input_start, input_end],
                        "stage_output_span": [output_start, out_len],
                        "alternatives": alternatives,
                    }
                )
                events.append(event)
            i = input_end

        return {"text": "".join(out), "protected": new_protected, "events": events}

    @staticmethod
    def _public_rule(row: dict[str, Any]) -> dict[str, Any]:
        return {
            "rule_id": row["rule_id"],
            "target_text": row["target_text"],
            "priority": row["priority"],
            "rule_type": row["rule_type"],
            "source_id": row["source_id"],
            "source_version_id": row["source_version_id"],
            "version_label": row.get("version_label"),
            "revision_id": row.get("revision_id"),
            "checksum_sha256": row.get("checksum_sha256"),
            "upstream_record_id": row.get("upstream_record_id"),
            "upstream_url": row.get("upstream_url"),
            "confidence": row.get("confidence"),
            "context": row.get("context"),
        }
