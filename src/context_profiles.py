"""Python reference adapter for the #46 profile contract; no localizer graph logic.

The production clients reuse the Rust model. This immutable reference snapshot
uses the same v1 format/validation for Python/Rust behavioral parity tests.
It neither loads files nor rewrites settings or dictionaries.
"""

from __future__ import annotations

import json
import re
import unicodedata
from dataclasses import dataclass
from types import MappingProxyType
from typing import Any, Iterable, Mapping

BUILT_INS = (
    ("general", "General"),
    ("technology-software", "Technology / Software"),
    ("banking-finance", "Banking / Finance"),
    ("business-marketing", "Business / Marketing"),
    ("legal", "Legal"),
    ("education", "Education"),
    ("government-public-administration", "Government / Public Administration"),
)


class ContextProfileError(ValueError):
    pass


def valid_id(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) <= 64
        and re.fullmatch(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*", value) is not None
    )


@dataclass(frozen=True)
class ContextProfile:
    context_id: str
    display_name: str
    parent_context_id: str | None = "general"
    built_in: bool = False
    enabled: bool = True


@dataclass(frozen=True, init=False)
class ContextProfiles:
    _profiles: Mapping[str, ContextProfile]

    def __init__(self, profiles: Iterable[ContextProfile] | None = None):
        if profiles is None:
            profiles = [
                ContextProfile(id, name, None if id == "general" else "general", True)
                for id, name in BUILT_INS
            ]
        items = list(profiles)
        if len(items) > 1024:
            raise ContextProfileError("Context profile document exceeds the size limit")
        indexed = {}
        reserved = {id for id, _ in BUILT_INS}
        for profile in items:
            if (
                not isinstance(profile, ContextProfile)
                or not valid_id(profile.context_id)
                or (
                    profile.parent_context_id is not None
                    and not valid_id(profile.parent_context_id)
                )
                or not isinstance(profile.display_name, str)
                or not profile.display_name
                or profile.display_name.strip() != profile.display_name
                or len(profile.display_name) > 128
                or any(
                    unicodedata.category(char) == "Cc" or 0xD800 <= ord(char) <= 0xDFFF
                    for char in profile.display_name
                )
                or type(profile.built_in) is not bool
                or type(profile.enabled) is not bool
            ):
                raise ContextProfileError("Invalid context profile fields")
            if profile.built_in != (profile.context_id in reserved):
                raise ContextProfileError(
                    "Reserved or unrecognized built-in context ID"
                )
            if profile.built_in and profile.parent_context_id != (
                None if profile.context_id == "general" else "general"
            ):
                raise ContextProfileError("Built-in parent links cannot be changed")
            if profile.context_id in indexed:
                raise ContextProfileError("Duplicate context ID")
            indexed[profile.context_id] = profile
        if not reserved.issubset(indexed):
            raise ContextProfileError("Missing built-in context")
        object.__setattr__(self, "_profiles", MappingProxyType(indexed))
        for id in indexed:
            self._chain(id, require_enabled=False)

    def _chain(self, id: str, *, require_enabled: bool) -> tuple[str, ...]:
        chain = []
        seen = set()
        while id is not None:
            if id in seen:
                raise ContextProfileError("Context inheritance cycle")
            seen.add(id)
            profile = self._profiles.get(id)
            if profile is None:
                raise ContextProfileError(f"Context does not exist: {id}")
            if require_enabled and not profile.enabled:
                raise ContextProfileError(f"Context is disabled: {id}")
            chain.append(id)
            id = profile.parent_context_id
        return tuple(chain)

    def resolve_chain(self, id: str) -> tuple[str, ...]:
        return self._chain(id, require_enabled=True)

    def resolve_request(self, context: dict[str, Any] | None) -> tuple[str, ...] | None:
        if not isinstance(context, dict) or "usage_context_id" not in context:
            return None
        id = context["usage_context_id"]
        if not isinstance(id, str) or not id:
            raise ContextProfileError(
                "usage_context_id must be a nonempty context ID string"
            )
        return self.resolve_chain(id)

    @classmethod
    def from_json(cls, data: bytes | str) -> "ContextProfiles":
        if len(data if isinstance(data, bytes) else data.encode("utf-8")) > 1024 * 1024:
            raise ContextProfileError("Context profile document exceeds the size limit")

        def unique(pairs):
            result = {}
            for key, value in pairs:
                if key in result:
                    raise ContextProfileError("Duplicate JSON field")
                result[key] = value
            return result

        value = json.loads(data, object_pairs_hook=unique)
        if not isinstance(value, dict) or set(value) != {"schema_version", "profiles"}:
            raise ContextProfileError("Invalid context profile document")
        if type(value["schema_version"]) is not int or value["schema_version"] != 1:
            raise ContextProfileError("Unsupported context profile schema version")
        if not isinstance(value["profiles"], list):
            raise ContextProfileError("Invalid profiles array")
        profiles = []
        for item in value["profiles"]:
            if (
                not isinstance(item, dict)
                or not {"context_id", "display_name", "built_in", "enabled"}.issubset(
                    item
                )
                or not set(item).issubset(
                    {
                        "context_id",
                        "display_name",
                        "parent_context_id",
                        "built_in",
                        "enabled",
                    }
                )
            ):
                raise ContextProfileError("Invalid context profile fields")
            profiles.append(ContextProfile(**item))
        return cls(profiles)


def context_level(metadata: Any, chain: tuple[str, ...] | None) -> int | None:
    neutral = len(chain) if chain is not None else 0
    if metadata is None:
        return neutral
    if not isinstance(metadata, dict):
        return None
    if "usage_context_id" in metadata:
        return None  # executable usage constraints belong inside constraints
    if "constraints" not in metadata:
        return neutral
    constraints = metadata["constraints"]
    if not isinstance(constraints, dict):
        return None
    if "usage_context_id" not in constraints:
        return neutral
    id = constraints["usage_context_id"]
    if not isinstance(id, str) or chain is None or id not in chain:
        return None
    return chain.index(id)


def context_selection(chain: tuple[str, ...], level: int) -> dict[str, Any]:
    matched = chain[level] if level < len(chain) else None
    return {
        "usage_context_id": chain[0],
        "matched_usage_context_id": matched,
        "context_distance": level if matched is not None else None,
        "context_level": "unscoped"
        if matched is None
        else "exact"
        if level == 0
        else "general"
        if matched == "general"
        else "parent",
    }
