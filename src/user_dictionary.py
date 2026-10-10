"""Local user dictionary storage for protected terms and explicit overrides."""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable
from private_store import prepare_store, schema_version


@dataclass(frozen=True)
class UserTerm:
    user_term_id: int
    kind: str
    source_text: str
    replacement: str | None
    source_locale: str | None
    target_locale: str | None
    priority: int
    enabled: bool
    note: str | None
    dictionary_id: str = "legacy"
    usage_context_id: str | None = None
    legacy: bool = True
    context_level: int = 0


def utc_now_iso() -> str:
    return (
        datetime.now(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z")
    )


class UserDictionary:
    def __init__(self, conn: sqlite3.Connection):
        self.conn = conn
        self.conn.row_factory = sqlite3.Row

    @classmethod
    def open(cls, path: Path, schema_path: Path) -> "UserDictionary":
        path.parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(path)
        conn.row_factory = sqlite3.Row
        try:
            prepare_store(conn)
            return cls(conn)
        except Exception:
            conn.close()
            raise

    def close(self) -> None:
        self.conn.close()

    def add_protected(
        self,
        source_text: str,
        *,
        source_locale: str | None = None,
        target_locale: str | None = None,
        priority: int = 0,
        note: str | None = None,
    ) -> int:
        return self._upsert(
            kind="protected",
            source_text=source_text,
            replacement=None,
            source_locale=source_locale,
            target_locale=target_locale,
            priority=priority,
            note=note,
        )

    def add_override(
        self,
        source_text: str,
        replacement: str,
        *,
        source_locale: str | None = None,
        target_locale: str | None = None,
        priority: int = 0,
        note: str | None = None,
    ) -> int:
        if not replacement:
            raise ValueError("override replacement must not be empty")
        return self._upsert(
            kind="override",
            source_text=source_text,
            replacement=replacement,
            source_locale=source_locale,
            target_locale=target_locale,
            priority=priority,
            note=note,
        )

    def _upsert(
        self,
        *,
        kind: str,
        source_text: str,
        replacement: str | None,
        source_locale: str | None,
        target_locale: str | None,
        priority: int,
        note: str | None,
    ) -> int:
        source_text = source_text.strip()
        if not source_text:
            raise ValueError("source_text must not be empty")
        now = utc_now_iso()
        row = self.conn.execute(
            """
            SELECT user_term_id FROM user_terms
            WHERE kind = ? AND source_text = ?
              AND dictionary_id = 'legacy' AND usage_context_id IS NULL
              AND ifnull(source_locale, '') = ifnull(?, '')
              AND ifnull(target_locale, '') = ifnull(?, '')
            """,
            (kind, source_text, source_locale, target_locale),
        ).fetchone()
        if row:
            term_id = int(row[0])
            self.conn.execute(
                """
                UPDATE user_terms
                SET replacement = ?, priority = ?, enabled = 1, note = ?, updated_at = ?
                WHERE user_term_id = ?
                """,
                (replacement, priority, note, now, term_id),
            )
        else:
            cursor = self.conn.execute(
                """
                INSERT INTO user_terms (
                    kind, source_text, replacement, source_locale, target_locale,
                    priority, enabled, note, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?)
                """,
                (
                    kind,
                    source_text,
                    replacement,
                    source_locale,
                    target_locale,
                    priority,
                    note,
                    now,
                    now,
                ),
            )
            term_id = int(cursor.lastrowid)
        self.conn.commit()
        return term_id

    def remove(self, user_term_id: int) -> bool:
        cursor = self.conn.execute(
            "DELETE FROM user_terms WHERE user_term_id = ?", (user_term_id,)
        )
        self.conn.commit()
        return bool(cursor.rowcount)

    def set_enabled(self, user_term_id: int, enabled: bool) -> bool:
        cursor = self.conn.execute(
            "UPDATE user_terms SET enabled = ?, updated_at = ? WHERE user_term_id = ?",
            (1 if enabled else 0, utc_now_iso(), user_term_id),
        )
        self.conn.commit()
        return bool(cursor.rowcount)

    def list_terms(self, *, enabled_only: bool = False) -> list[UserTerm]:
        sql = "SELECT * FROM user_terms"
        if enabled_only:
            sql += " WHERE enabled = 1"
        sql += " ORDER BY kind, source_text, priority DESC, user_term_id"
        rows = self.conn.execute(sql).fetchall()
        return [self._row_to_term(row) for row in rows]

    def candidates(
        self, source_locale: str, target_locale: str, chain=None
    ) -> list[UserTerm]:
        if schema_version(self.conn) == "2":
            rows = self.conn.execute(
                """SELECT t.* FROM user_terms t JOIN user_dictionaries d USING(dictionary_id)
                WHERE t.enabled=1 AND d.enabled=1 AND (source_locale IS NULL OR source_locale=?)
                AND (target_locale IS NULL OR target_locale=?)""",
                (source_locale, target_locale),
            ).fetchall()
            result = []
            from dataclasses import replace

            for row in rows:
                term = self._row_to_term(row)
                if term.usage_context_id is None:
                    level = len(chain) if chain else 0
                elif chain and term.usage_context_id in chain:
                    level = chain.index(term.usage_context_id)
                else:
                    continue
                result.append(replace(term, context_level=level))
            return result
        rows = self.conn.execute(
            """
            SELECT * FROM user_terms
            WHERE enabled = 1
              AND (source_locale IS NULL OR source_locale = ?)
              AND (target_locale IS NULL OR target_locale = ?)
            ORDER BY length(source_text) DESC, source_text, priority DESC, user_term_id
            """,
            (source_locale, target_locale),
        ).fetchall()
        return [self._row_to_term(row) for row in rows]

    @staticmethod
    def _row_to_term(row: sqlite3.Row) -> UserTerm:
        return UserTerm(
            user_term_id=int(row["user_term_id"]),
            kind=str(row["kind"]),
            source_text=str(row["source_text"]),
            replacement=row["replacement"],
            source_locale=row["source_locale"],
            target_locale=row["target_locale"],
            priority=int(row["priority"]),
            enabled=bool(row["enabled"]),
            note=row["note"],
            dictionary_id=row["dictionary_id"]
            if "dictionary_id" in row.keys()
            else "legacy",
            usage_context_id=row["usage_context_id"]
            if "usage_context_id" in row.keys()
            else None,
            legacy=bool(row["legacy"]) if "legacy" in row.keys() else True,
        )


def specificity(term: UserTerm) -> int:
    return int(term.source_locale is not None) + int(term.target_locale is not None)


def group_by_surface(terms: Iterable[UserTerm]) -> dict[str, list[UserTerm]]:
    result: dict[str, list[UserTerm]] = {}
    for term in terms:
        result.setdefault(term.source_text, []).append(term)
    return result
