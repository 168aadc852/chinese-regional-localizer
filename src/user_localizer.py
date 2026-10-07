"""User-local control layer around the deterministic shared localizer.

User rules live in a separate SQLite database and are applied before shared
entities/OpenCC rules. Matched user spans are never passed to the shared engine.
"""

from __future__ import annotations

from typing import Any

from localizer_engine import LocalizerEngine, PrefixMatcher, ROUTES, UnsupportedRouteError
from user_dictionary import UserDictionary, UserTerm, group_by_surface, specificity


class UserControlledLocalizer:
    def __init__(self, shared_engine: LocalizerEngine, user_dictionary: UserDictionary):
        self.shared_engine = shared_engine
        self.user_dictionary = user_dictionary

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

        terms = self.user_dictionary.candidates(source_locale, target_locale)
        by_surface = group_by_surface(terms)
        matcher = PrefixMatcher(by_surface)
        segments = self._segments(text, matcher, by_surface)

        output_parts: list[str] = []
        events: list[dict[str, Any]] = []
        output_cursor = 0

        for segment in segments:
            start, end = segment["input_span"]
            source_text = text[start:end]
            if segment["kind"] == "shared":
                result = self.shared_engine.localize(
                    source_text,
                    source_locale,
                    target_locale,
                    context=context,
                )
                replacement = result["output"]
                for event in result["changes"]:
                    copied = dict(event)
                    self._offset_span(copied, "original_input_span", start)
                    self._offset_span(copied, "final_output_span", output_cursor)
                    copied["user_layer_segment_input_span"] = [start, end]
                    events.append(copied)
            else:
                replacement, event = self._resolve_user_segment(
                    source_text,
                    segment["terms"],
                    source_locale,
                    target_locale,
                    [start, end],
                    output_cursor,
                )
                events.append(event)

            output_parts.append(replacement)
            output_cursor += len(replacement)

        output = "".join(output_parts)
        return {
            "input": text,
            "output": output,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "route": [f"{a}->{b}" for a, b in route],
            "changes": events,
            "review_needed": any(event.get("review_needed", False) for event in events),
            "user_dictionary_applied": any(
                event.get("type", "").startswith("user_") for event in events
            ),
        }

    @staticmethod
    def _offset_span(event: dict[str, Any], key: str, offset: int) -> None:
        value = event.get(key)
        if isinstance(value, list) and len(value) == 2:
            event[key] = [int(value[0]) + offset, int(value[1]) + offset]

    def _segments(
        self,
        text: str,
        matcher: PrefixMatcher,
        by_surface: dict[str, list[UserTerm]],
    ) -> list[dict[str, Any]]:
        if not text:
            return []
        segments: list[dict[str, Any]] = []
        shared_start = 0
        i = 0
        while i < len(text):
            matches = matcher.matches(text, i)
            if not matches:
                i += 1
                continue
            surface = matches[0]
            if shared_start < i:
                segments.append(
                    {"kind": "shared", "input_span": [shared_start, i], "terms": []}
                )
            end = i + len(surface)
            segments.append(
                {
                    "kind": "user",
                    "input_span": [i, end],
                    "terms": by_surface[surface],
                }
            )
            i = end
            shared_start = end
        if shared_start < len(text):
            segments.append(
                {"kind": "shared", "input_span": [shared_start, len(text)], "terms": []}
            )
        if not segments:
            return [{"kind": "shared", "input_span": [0, len(text)], "terms": []}]
        return segments

    def _resolve_user_segment(
        self,
        source_text: str,
        terms: list[UserTerm],
        source_locale: str,
        target_locale: str,
        input_span: list[int],
        output_start: int,
    ) -> tuple[str, dict[str, Any]]:
        protected = [term for term in terms if term.kind == "protected"]
        if protected:
            winner = self._rank(protected)[0]
            return source_text, {
                "type": "user_protected",
                "applied": False,
                "review_needed": False,
                "reason": "protected_by_user",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": [output_start, output_start + len(source_text)],
                "user_term_id": winner.user_term_id,
                "note": winner.note,
                "provenance": "user_dictionary",
            }

        ranked = self._rank([term for term in terms if term.kind == "override"])
        if not ranked:
            return source_text, {
                "type": "user_override",
                "applied": False,
                "review_needed": True,
                "reason": "invalid_user_rule",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": [output_start, output_start + len(source_text)],
                "provenance": "user_dictionary",
            }

        top_rank = (specificity(ranked[0]), ranked[0].priority)
        top = [
            term
            for term in ranked
            if (specificity(term), term.priority) == top_rank
        ]
        replacements = {term.replacement for term in top}
        if len(replacements) != 1:
            return source_text, {
                "type": "user_override",
                "applied": False,
                "review_needed": True,
                "reason": "ambiguous_user_override",
                "original": source_text,
                "replacement": source_text,
                "source_locale": source_locale,
                "target_locale": target_locale,
                "original_input_span": input_span,
                "final_output_span": [output_start, output_start + len(source_text)],
                "candidates": [
                    {
                        "user_term_id": term.user_term_id,
                        "replacement": term.replacement,
                        "source_locale": term.source_locale,
                        "target_locale": term.target_locale,
                        "priority": term.priority,
                    }
                    for term in top
                ],
                "provenance": "user_dictionary",
            }

        winner = top[0]
        replacement = winner.replacement or source_text
        return replacement, {
            "type": "user_override",
            "applied": replacement != source_text,
            "review_needed": False,
            "reason": "user_fixed_override",
            "original": source_text,
            "replacement": replacement,
            "source_locale": source_locale,
            "target_locale": target_locale,
            "original_input_span": input_span,
            "final_output_span": [output_start, output_start + len(replacement)],
            "user_term_id": winner.user_term_id,
            "note": winner.note,
            "provenance": "user_dictionary",
        }

    @staticmethod
    def _rank(terms: list[UserTerm]) -> list[UserTerm]:
        return sorted(
            terms,
            key=lambda term: (
                -specificity(term),
                -term.priority,
                term.user_term_id,
            ),
        )
