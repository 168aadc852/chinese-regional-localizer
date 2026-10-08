"""Deterministic reference localization engine for Phase 2C.

The engine resolves conservative high-confidence entity surfaces first, protects
resolved/review-required entity spans, then applies staged terminology rules.
It caches compiled matchers per engine instance and keeps original-to-final span
alignment for explainable UI highlighting.
"""

from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from typing import Any, Iterable
from context_profiles import ContextProfiles, context_level, context_selection


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


@dataclass(frozen=True)
class AlignmentSpan:
    start: int
    end: int
    original_start: int
    original_end: int


class PrefixMatcher:
    """Small deterministic trie used for longest-prefix matching."""

    _END = "\0"

    def __init__(self, values: Iterable[str]):
        self.root: dict[str, Any] = {}
        for value in set(values):
            if not value:
                continue
            node = self.root
            for char in value:
                node = node.setdefault(char, {})
            node[self._END] = value

    def matches(self, text: str, start: int, limit: int | None = None) -> list[str]:
        node = self.root
        found: list[str] = []
        end_limit = len(text) if limit is None else min(limit, len(text))
        cursor = start
        while cursor < end_limit:
            char = text[cursor]
            child = node.get(char)
            if child is None:
                break
            node = child
            cursor += 1
            value = node.get(self._END)
            if value is not None:
                found.append(value)
        found.sort(key=lambda value: (-len(value), value))
        return found


class LocalizerEngine:
    def __init__(self, conn: sqlite3.Connection, *, context_profiles: ContextProfiles | None = None):
        self.conn = conn
        self._context_profiles = context_profiles if context_profiles is not None else ContextProfiles()
        if not isinstance(self._context_profiles, ContextProfiles):
            raise TypeError("context_profiles must be a validated ContextProfiles snapshot")
        self.conn.row_factory = sqlite3.Row
        self._entity_cache: dict[
            str, tuple[dict[str, list[dict[str, Any]]], PrefixMatcher]
        ] = {}
        self._term_cache: dict[
            tuple[str, str], tuple[dict[str, list[dict[str, Any]]], PrefixMatcher]
        ] = {}
        self._has_current_versions = self._column_exists("source_versions", "is_current")

    def _column_exists(self, table: str, column: str) -> bool:
        rows = self.conn.execute(f"PRAGMA table_info({table})").fetchall()
        return any(row["name"] == column for row in rows)

    def clear_cache(self) -> None:
        """Clear compiled indexes after mutating the underlying database."""
        self._entity_cache.clear()
        self._term_cache.clear()

    def localize(
        self,
        text: str,
        source_locale: str,
        target_locale: str,
        *,
        context: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        route = ROUTES.get((source_locale, target_locale))
        if route is None:
            raise UnsupportedRouteError(f"Unsupported route: {source_locale} -> {target_locale}")
        usage_chain = self.validate_usage_context(context)

        initial_alignment = (
            [AlignmentSpan(0, len(text), 0, len(text))] if text else []
        )
        entity_result = self._apply_entities(
            text, source_locale, target_locale, initial_alignment
        )
        current_text = entity_result["text"]
        protected = entity_result["protected"]
        alignment = entity_result["alignment"]
        events = list(entity_result["events"])

        for stage_source, stage_target in route:
            stage = self._apply_term_stage(
                current_text,
                protected,
                alignment,
                stage_source,
                stage_target,
                context=context or {},
                usage_chain=usage_chain if stage_target in ("zh-HK", "zh-TW") else None,
            )
            current_text = stage["text"]
            protected = stage["protected"]
            alignment = stage["alignment"]
            events.extend(stage["events"])

        for event in events:
            original_span = event.get("original_input_span")
            if original_span:
                event["final_output_span"] = list(
                    self._final_output_span(
                        alignment, int(original_span[0]), int(original_span[1])
                    )
                )

        return {
            "input": text,
            "output": current_text,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "route": [f"{a}->{b}" for a, b in route],
            "changes": events,
            "review_needed": any(event.get("review_needed", False) for event in events),
        }

    def validate_usage_context(self, context: dict[str, Any] | None) -> tuple[str, ...] | None:
        return self._context_profiles.resolve_request(context)

    def _current_evidence_clause(self, alias: str) -> str:
        if not self._has_current_versions:
            return "1 = 1"
        return (
            f"({alias}.source_version_id IS NULL OR EXISTS ("
            "SELECT 1 FROM source_versions csv "
            f"WHERE csv.source_version_id = {alias}.source_version_id AND csv.is_current = 1))"
        )

    def _name_has_current_evidence_clause(self, name_alias: str) -> str:
        if not self._has_current_versions:
            return "1 = 1"
        return f"""
            (
                NOT EXISTS (
                    SELECT 1 FROM name_evidence any_ne
                    WHERE any_ne.localized_name_id = {name_alias}.localized_name_id
                )
                OR EXISTS (
                    SELECT 1
                    FROM name_evidence current_ne
                    LEFT JOIN source_versions current_sv
                      ON current_sv.source_version_id = current_ne.source_version_id
                    WHERE current_ne.localized_name_id = {name_alias}.localized_name_id
                      AND (
                          current_ne.source_version_id IS NULL
                          OR current_sv.is_current = 1
                      )
                )
            )
        """

    def _entity_index(self, source_locale: str) -> dict[str, list[dict[str, Any]]]:
        cached = self._entity_cache.get(source_locale)
        if cached is not None:
            return cached[0]

        evidence_clause = self._name_has_current_evidence_clause("ln")
        rows = self.conn.execute(
            f"""
            SELECT ln.localized_name_id, ln.concept_id, ln.text, ln.name_type,
                   ln.confidence, c.concept_type,
                   (SELECT external_value FROM external_ids e
                    WHERE e.concept_id = c.concept_id AND e.namespace = 'wikidata'
                    LIMIT 1) AS qid
            FROM localized_names ln
            JOIN concepts c ON c.concept_id = ln.concept_id
            WHERE ln.locale = ? AND ln.domain = 'entity'
              AND {evidence_clause}
            """,
            (source_locale,),
        ).fetchall()
        index: dict[str, dict[int, dict[str, Any]]] = {}
        for row in rows:
            candidate = dict(row)
            if not self._safe_entity_surface(
                candidate["text"], candidate["name_type"]
            ):
                continue
            bucket = index.setdefault(row["text"], {})
            concept_id = int(row["concept_id"])
            current = bucket.get(concept_id)
            if current is None or (candidate.get("confidence") or 0) > (
                current.get("confidence") or 0
            ):
                bucket[concept_id] = candidate
        final = {text: list(by_concept.values()) for text, by_concept in index.items()}
        self._entity_cache[source_locale] = (final, PrefixMatcher(final))
        return final

    def _entity_matcher(self, source_locale: str) -> PrefixMatcher:
        if source_locale not in self._entity_cache:
            self._entity_index(source_locale)
        return self._entity_cache[source_locale][1]

    @staticmethod
    def _safe_entity_surface(text: str, name_type: str) -> bool:
        surface = text.strip()
        if surface != text or not surface:
            return False
        has_cjk = any("\u3400" <= ch <= "\u9fff" for ch in surface)
        punctuation = any(ch in "·•・-–—()（）[]【】/\\ " for ch in surface)
        has_ascii = any(ch.isascii() and ch.isalnum() for ch in surface)
        if has_cjk:
            pure_cjk = all("\u3400" <= ch <= "\u9fff" for ch in surface)
            minimum = 4 if name_type == "alias" and pure_cjk else 3
            if punctuation or has_ascii:
                minimum = 2
            return len(surface) >= minimum
        return len(surface) >= 4

    @staticmethod
    def _boundary_ok(text: str, start: int, end: int, surface: str) -> bool:
        if not surface:
            return False
        if surface[0].isascii() and surface[0].isalnum():
            if start > 0 and text[start - 1].isascii() and text[start - 1].isalnum():
                return False
        if surface[-1].isascii() and surface[-1].isalnum():
            if end < len(text) and text[end].isascii() and text[end].isalnum():
                return False
        return True

    def _target_entity_names(self, concept_id: int, target_locale: str) -> list[sqlite3.Row]:
        evidence_clause = self._name_has_current_evidence_clause("ln")
        return self.conn.execute(
            f"""
            SELECT ln.localized_name_id, ln.text, ln.confidence
            FROM localized_names ln
            WHERE ln.concept_id = ? AND ln.locale = ?
              AND ln.name_type = 'preferred' AND ln.is_preferred = 1
              AND {evidence_clause}
            ORDER BY ln.confidence DESC, ln.text
            """,
            (concept_id, target_locale),
        ).fetchall()

    def _name_evidence(self, localized_name_id: int) -> list[dict[str, Any]]:
        current_clause = self._current_evidence_clause("ne")
        rows = self.conn.execute(
            f"""
            SELECT ne.source_id, ne.upstream_record_id, ne.upstream_url,
                   ne.upstream_revision, ne.evidence_type, ne.confidence,
                   ne.retrieved_at, sv.version_label, sv.revision_id,
                   sv.checksum_sha256
            FROM name_evidence ne
            LEFT JOIN source_versions sv ON sv.source_version_id = ne.source_version_id
            WHERE ne.localized_name_id = ? AND {current_clause}
            ORDER BY ne.evidence_id
            """,
            (localized_name_id,),
        ).fetchall()
        return [dict(row) for row in rows]

    def _apply_entities(
        self,
        text: str,
        source_locale: str,
        target_locale: str,
        alignment: list[AlignmentSpan],
    ) -> dict[str, Any]:
        entity_index = self._entity_index(source_locale)
        matcher = self._entity_matcher(source_locale)
        out: list[str] = []
        protected: list[ProtectedSpan] = []
        new_alignment: list[AlignmentSpan] = []
        events: list[dict[str, Any]] = []
        i = 0
        out_len = 0

        while i < len(text):
            source_text = None
            for key in matcher.matches(text, i):
                end = i + len(key)
                if self._boundary_ok(text, i, end, key):
                    source_text = key
                    break
            if source_text is None:
                out.append(text[i])
                self._copy_alignment_range(
                    alignment, i, i + 1, out_len, new_alignment
                )
                out_len += 1
                i += 1
                continue

            concepts = entity_index[source_text]
            input_start = i
            input_end = i + len(source_text)
            original_start, original_end = self._original_span_for_current(
                alignment, input_start, input_end
            )
            output_start = out_len

            if len(concepts) != 1:
                replacement = source_text
                out.append(replacement)
                out_len += len(replacement)
                self._append_alignment(
                    new_alignment,
                    AlignmentSpan(output_start, out_len, original_start, original_end),
                )
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
                        "original_input_span": [original_start, original_end],
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
            target_names = self._target_entity_names(
                int(concept["concept_id"]), target_locale
            )
            distinct_targets = {row["text"] for row in target_names}
            if len(distinct_targets) != 1:
                replacement = source_text
                out.append(replacement)
                out_len += len(replacement)
                self._append_alignment(
                    new_alignment,
                    AlignmentSpan(output_start, out_len, original_start, original_end),
                )
                protected.append(ProtectedSpan(output_start, out_len))
                events.append(
                    {
                        "type": "entity",
                        "applied": False,
                        "review_needed": True,
                        "reason": "missing_target_name"
                        if not target_names
                        else "ambiguous_target_name",
                        "original": source_text,
                        "replacement": source_text,
                        "source_locale": source_locale,
                        "target_locale": target_locale,
                        "input_span": [input_start, input_end],
                        "output_span": [output_start, out_len],
                        "original_input_span": [original_start, original_end],
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
            self._append_alignment(
                new_alignment,
                AlignmentSpan(output_start, out_len, original_start, original_end),
            )
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
                    "original_input_span": [original_start, original_end],
                    "concept_id": concept["concept_id"],
                    "concept_type": concept["concept_type"],
                    "qid": concept["qid"],
                    "confidence": target_row["confidence"],
                    "evidence": self._name_evidence(
                        int(target_row["localized_name_id"])
                    ),
                }
            )
            i = input_end

        return {
            "text": "".join(out),
            "protected": protected,
            "alignment": new_alignment,
            "events": events,
        }

    def _term_rule_index(
        self, source_locale: str, target_locale: str
    ) -> dict[str, list[dict[str, Any]]]:
        cache_key = (source_locale, target_locale)
        cached = self._term_cache.get(cache_key)
        if cached is not None:
            return cached[0]

        current_filter = ""
        if self._has_current_versions:
            current_filter = "AND (tr.source_version_id IS NULL OR sv.is_current = 1)"
        rows = self.conn.execute(
            f"""
            SELECT tr.rule_id, tr.source_text, tr.target_text, tr.priority,
                   tr.context_constraint, tr.source_id, tr.source_version_id,
                   tr.upstream_record_id, tr.upstream_url, tr.confidence,
                   tr.rule_type, tr.domain, sv.version_label, sv.revision_id,
                   sv.checksum_sha256
            FROM term_rules tr
            LEFT JOIN source_versions sv ON sv.source_version_id = tr.source_version_id
            WHERE tr.source_locale = ? AND tr.target_locale = ? AND tr.active = 1
              {current_filter}
            ORDER BY tr.source_text, tr.priority DESC, tr.rule_id
            """,
            (source_locale, target_locale),
        ).fetchall()
        index: dict[str, list[dict[str, Any]]] = {}
        for row in rows:
            item = dict(row)
            item["context_parse_error"] = False
            if item.get("context_constraint") is not None:
                try:
                    item["context"] = json.loads(item["context_constraint"])
                    if not isinstance(item["context"], dict):
                        item["context_parse_error"] = True
                except json.JSONDecodeError:
                    item["context"] = None
                    item["context_parse_error"] = True
            else:
                item["context"] = None
            index.setdefault(row["source_text"], []).append(item)
        self._term_cache[cache_key] = (index, PrefixMatcher(index))
        return index

    def _term_matcher(self, source_locale: str, target_locale: str) -> PrefixMatcher:
        key = (source_locale, target_locale)
        if key not in self._term_cache:
            self._term_rule_index(source_locale, target_locale)
        return self._term_cache[key][1]

    @staticmethod
    def _value_list(value: Any) -> list[str]:
        if isinstance(value, str):
            return [value]
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            return value
        return []

    def _rule_context_allows(
        self,
        row: dict[str, Any],
        text: str,
        start: int,
        end: int,
        runtime_context: dict[str, Any],
    ) -> bool:
        if row.get("context_parse_error"):
            return False
        metadata = row.get("context")
        if not isinstance(metadata, dict):
            return True
        constraints = metadata.get("constraints")
        if constraints is None:
            return True
        if not isinstance(constraints, dict):
            return False

        domains = self._value_list(constraints.get("domain"))
        if domains and runtime_context.get("domain") not in domains:
            return False

        checks = (
            ("preceded_by", text[:start].endswith, True),
            ("followed_by", text[end:].startswith, True),
            ("not_preceded_by", text[:start].endswith, False),
            ("not_followed_by", text[end:].startswith, False),
        )
        for key, predicate, expected in checks:
            values = self._value_list(constraints.get(key))
            if not values:
                continue
            matched = any(predicate(value) for value in values)
            if matched != expected:
                return False

        if constraints.get("word_boundary") is True:
            if not self._boundary_ok(text, start, end, row["source_text"]):
                return False
        return True

    def _apply_term_stage(
        self,
        text: str,
        protected: list[ProtectedSpan],
        alignment: list[AlignmentSpan],
        source_locale: str,
        target_locale: str,
        *,
        context: dict[str, Any],
        usage_chain: tuple[str, ...] | None = None,
    ) -> dict[str, Any]:
        rule_index = self._term_rule_index(source_locale, target_locale)
        matcher = self._term_matcher(source_locale, target_locale)
        protected = sorted(protected, key=lambda span: span.start)
        span_cursor = 0
        out: list[str] = []
        new_protected: list[ProtectedSpan] = []
        new_alignment: list[AlignmentSpan] = []
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
                self._copy_alignment_range(
                    alignment, span.start, span.end, start, new_alignment
                )
                out_len += len(chunk)
                new_protected.append(ProtectedSpan(start, out_len))
                i = span.end
                span_cursor += 1
                continue

            next_protected_start = (
                protected[span_cursor].start if span_cursor < len(protected) else len(text)
            )
            source_text: str | None = None
            ranked_rows: list[tuple[dict[str, Any], int]] = []
            for key in matcher.matches(text, i, next_protected_start):
                key_end = i + len(key)
                applicable = []
                for row in rule_index[key]:
                    level = context_level(row.get("context"), usage_chain)
                    if level is not None and self._rule_context_allows(
                        row, text, i, key_end, context
                    ):
                        applicable.append((row, level))
                if applicable:
                    source_text = key
                    ranked_rows = applicable
                    break

            if source_text is None:
                out.append(text[i])
                self._copy_alignment_range(
                    alignment, i, i + 1, out_len, new_alignment
                )
                out_len += 1
                i += 1
                continue

            level = min(level for _, level in ranked_rows)
            rows = [row for row, rank in ranked_rows if rank == level]
            selected_context = (
                context_selection(usage_chain, level) if usage_chain is not None else None
            )
            max_priority = max(int(row["priority"]) for row in rows)
            winners = [row for row in rows if int(row["priority"]) == max_priority]
            winner_targets = {row["target_text"] for row in winners}
            input_start = i
            input_end = i + len(source_text)
            original_start, original_end = self._original_span_for_current(
                alignment, input_start, input_end
            )
            output_start = out_len

            if len(winner_targets) != 1:
                out.append(source_text)
                out_len += len(source_text)
                self._append_alignment(
                    new_alignment,
                    AlignmentSpan(output_start, out_len, original_start, original_end),
                )
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
                        "original_input_span": [original_start, original_end],
                        "candidates": [self._public_rule(row) for row in winners],
                    }
                )
                if selected_context is not None:
                    events[-1]["context_selection"] = selected_context
                i = input_end
                continue

            winner = winners[0]
            replacement = winner["target_text"]
            out.append(replacement)
            out_len += len(replacement)
            self._append_alignment(
                new_alignment,
                AlignmentSpan(output_start, out_len, original_start, original_end),
            )

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
                        "original_input_span": [original_start, original_end],
                        "alternatives": alternatives,
                    }
                )
                events.append(event)
                if selected_context is not None:
                    event["context_selection"] = selected_context
            i = input_end

        return {
            "text": "".join(out),
            "protected": new_protected,
            "alignment": new_alignment,
            "events": events,
        }

    @staticmethod
    def _append_alignment(
        output: list[AlignmentSpan], span: AlignmentSpan
    ) -> None:
        if span.start == span.end:
            return
        if output:
            previous = output[-1]
            linear_previous = (previous.end - previous.start) == (
                previous.original_end - previous.original_start
            )
            linear_span = (span.end - span.start) == (
                span.original_end - span.original_start
            )
            same_replacement_origin = (
                previous.end == span.start
                and previous.original_start == span.original_start
                and previous.original_end == span.original_end
            )
            contiguous_linear = (
                previous.end == span.start
                and previous.original_end == span.original_start
                and linear_previous
                and linear_span
            )
            if same_replacement_origin or contiguous_linear:
                output[-1] = AlignmentSpan(
                    previous.start,
                    span.end,
                    previous.original_start,
                    span.original_end if contiguous_linear else previous.original_end,
                )
                return
        output.append(span)

    @classmethod
    def _copy_alignment_range(
        cls,
        alignment: list[AlignmentSpan],
        start: int,
        end: int,
        new_start: int,
        output: list[AlignmentSpan],
    ) -> None:
        for span in alignment:
            overlap_start = max(start, span.start)
            overlap_end = min(end, span.end)
            if overlap_start >= overlap_end:
                continue
            copied_start = new_start + (overlap_start - start)
            copied_end = new_start + (overlap_end - start)
            current_len = span.end - span.start
            original_len = span.original_end - span.original_start
            if current_len == original_len:
                original_start = span.original_start + (overlap_start - span.start)
                original_end = span.original_start + (overlap_end - span.start)
            else:
                original_start = span.original_start
                original_end = span.original_end
            cls._append_alignment(
                output,
                AlignmentSpan(
                    copied_start,
                    copied_end,
                    original_start,
                    original_end,
                ),
            )

    @staticmethod
    def _original_span_for_current(
        alignment: list[AlignmentSpan], start: int, end: int
    ) -> tuple[int, int]:
        original_ranges: list[tuple[int, int]] = []
        for span in alignment:
            overlap_start = max(start, span.start)
            overlap_end = min(end, span.end)
            if overlap_start >= overlap_end:
                continue
            current_len = span.end - span.start
            original_len = span.original_end - span.original_start
            if current_len == original_len:
                original_ranges.append(
                    (
                        span.original_start + (overlap_start - span.start),
                        span.original_start + (overlap_end - span.start),
                    )
                )
            else:
                original_ranges.append((span.original_start, span.original_end))
        if not original_ranges:
            return start, end
        return (
            min(item[0] for item in original_ranges),
            max(item[1] for item in original_ranges),
        )

    @staticmethod
    def _final_output_span(
        alignment: list[AlignmentSpan], original_start: int, original_end: int
    ) -> tuple[int, int]:
        ranges: list[tuple[int, int]] = []
        for span in alignment:
            overlap_start = max(original_start, span.original_start)
            overlap_end = min(original_end, span.original_end)
            if overlap_start >= overlap_end:
                continue
            current_len = span.end - span.start
            original_len = span.original_end - span.original_start
            if current_len == original_len:
                ranges.append(
                    (
                        span.start + (overlap_start - span.original_start),
                        span.start + (overlap_end - span.original_start),
                    )
                )
            else:
                ranges.append((span.start, span.end))
        if not ranges:
            return original_start, original_end
        return min(item[0] for item in ranges), max(item[1] for item in ranges)

    @staticmethod
    def _public_rule(row: dict[str, Any]) -> dict[str, Any]:
        return {
            "rule_id": row["rule_id"],
            "target_text": row["target_text"],
            "priority": row["priority"],
            "rule_type": row["rule_type"],
            "domain": row.get("domain"),
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
