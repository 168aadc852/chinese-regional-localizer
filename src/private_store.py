"""Versioned local My Dictionaries storage; no network or shared-data writes."""

from __future__ import annotations

import csv
import io
import sqlite3
from pathlib import Path

from context_profiles import ContextProfiles

ROOT = Path(__file__).resolve().parents[1]
LOCALES = {"zh-CN", "zh-Hant", "zh-HK", "zh-TW"}


def schema_version(conn):
    has_metadata = conn.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='user_metadata'"
    ).fetchone()
    if not has_metadata:
        columns = [row[1] for row in conn.execute("PRAGMA table_info(user_terms)")]
        if columns == [
            "user_term_id",
            "kind",
            "source_text",
            "replacement",
            "source_locale",
            "target_locale",
            "priority",
            "enabled",
            "note",
            "created_at",
            "updated_at",
        ]:
            return "legacy-unversioned"
        raise ValueError("Unrecognized unversioned private store")
    version = conn.execute(
        "SELECT value FROM user_metadata WHERE key='schema_version'"
    ).fetchone()
    if version is None or version[0] not in ("0.1", "2"):
        raise ValueError("Unsupported private-store schema version")
    return version[0]


def _execute_sql(conn, script):
    # executescript implicitly commits in Python: execute each DDL inside our transaction instead.
    statement = ""
    for line in script.splitlines(keepends=True):
        statement += line
        if sqlite3.complete_statement(statement):
            conn.execute(statement)
            statement = ""
    if statement.strip():
        raise ValueError("Incomplete private schema SQL")


def prepare_store(conn):
    """Atomic/idempotent creation or non-destructive legacy upgrade."""
    if conn.in_transaction:
        raise ValueError("Private-store open requires an idle connection")
    conn.execute("PRAGMA foreign_keys=ON")
    tables = {
        row[0]
        for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
    }
    if not tables:
        script = (ROOT / "schema/user-dictionary-v2.sql").read_text(encoding="utf-8")
    else:
        version = schema_version(conn)
        if version == "2":
            validate_store(conn)
            return
        extension = conn.execute("""SELECT name FROM sqlite_master WHERE type='view'
            OR (type='trigger' AND tbl_name='user_terms')
            OR (type='table' AND name NOT IN ('user_terms','user_metadata') AND name NOT LIKE 'sqlite_%') LIMIT 1""").fetchone()
        if extension:
            raise ValueError(
                "Unsupported legacy private-store extension; original store retained"
            )
        script = (ROOT / "schema/private-migrations/0001-v2.sql").read_text(
            encoding="utf-8"
        )
    with conn:
        conn.execute("BEGIN IMMEDIATE")
        count = (
            conn.execute("SELECT count(*) FROM user_terms").fetchone()[0]
            if tables
            else 0
        )
        if tables and version == "legacy-unversioned":
            conn.execute(
                "CREATE TABLE user_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL)"
            )
            conn.execute("INSERT INTO user_metadata VALUES ('schema_version','0.1')")
        _execute_sql(conn, script)
        if conn.execute("SELECT count(*) FROM user_terms").fetchone()[0] != count:
            raise ValueError("Private migration count mismatch")
        validate_store(conn)


def validate_store(conn):
    if (
        conn.execute("PRAGMA quick_check").fetchone()[0] != "ok"
        or conn.execute("PRAGMA foreign_key_check").fetchone()
    ):
        raise ValueError("Invalid private store")
    for id in ("personal", "legacy"):
        row = conn.execute(
            "SELECT system_managed FROM user_dictionaries WHERE dictionary_id=?", (id,)
        ).fetchone()
        if row is None or row[0] != 1:
            raise ValueError("Missing reserved private dictionary")
    conn.execute("SELECT dictionary_id,usage_context_id,legacy FROM user_terms LIMIT 0")


class PrivateStore:
    def __init__(self, conn, profiles=None):
        self.conn = conn
        self.profiles = profiles if profiles is not None else ContextProfiles()
        if not isinstance(self.profiles, ContextProfiles):
            raise TypeError("profiles must be a validated ContextProfiles snapshot")
        prepare_store(conn)

    @classmethod
    def open(cls, path, profiles=None):
        conn = sqlite3.connect(path)
        try:
            return cls(conn, profiles)
        except Exception:
            conn.close()
            raise

    def close(self):
        self.conn.close()

    def create_dictionary(self, id, name):
        if not id.strip() or not name.strip():
            raise ValueError("Dictionary identity and name are required")
        with self.conn:
            self.conn.execute(
                "INSERT INTO user_dictionaries(dictionary_id,name) VALUES (?,?)",
                (id, name),
            )

    def rename_dictionary(self, id, name):
        if not name.strip():
            raise ValueError("Dictionary name is required")
        self._mutable_dictionary(id)
        with self.conn:
            self.conn.execute(
                "UPDATE user_dictionaries SET name=? WHERE dictionary_id=?", (name, id)
            )

    def _mutable_dictionary(self, id):
        row = self.conn.execute(
            "SELECT system_managed FROM user_dictionaries WHERE dictionary_id=?", (id,)
        ).fetchone()
        if row is None or row[0]:
            raise ValueError("Unknown or reserved dictionary")

    def delete_dictionary(self, id):
        self._mutable_dictionary(id)
        with self.conn:
            self.conn.execute("DELETE FROM user_terms WHERE dictionary_id=?", (id,))
            self.conn.execute(
                "DELETE FROM user_dictionaries WHERE dictionary_id=?", (id,)
            )

    def set_dictionary_enabled(self, id, enabled):
        with self.conn:
            if not self.conn.execute(
                "UPDATE user_dictionaries SET enabled=? WHERE dictionary_id=?",
                (int(enabled), id),
            ).rowcount:
                raise ValueError("Unknown dictionary")

    def _validate_term(self, source, replacement, target, usage, source_locale=None):
        if not source.strip() or not replacement or target not in LOCALES:
            raise ValueError("Source, preferred text and target locale are required")
        if source_locale is not None and source_locale not in LOCALES:
            raise ValueError("Invalid source locale")
        if usage is not None:
            self.profiles.resolve_chain(usage)

    def _put(self, dictionary, source, replacement, target, usage, source_locale, note):
        self._validate_term(source, replacement, target, usage, source_locale)
        self.conn.execute(
            """INSERT INTO user_terms(kind,source_text,replacement,source_locale,target_locale,
            priority,enabled,note,created_at,updated_at,dictionary_id,usage_context_id,legacy)
            VALUES ('override',?,?,?,?,0,1,?,strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'),?,?,0)
            ON CONFLICT DO UPDATE SET replacement=excluded.replacement,enabled=1,note=excluded.note,updated_at=excluded.updated_at,legacy=0,priority=0""",
            (source, replacement, source_locale, target, note, dictionary, usage),
        )
        return self.conn.execute(
            """SELECT user_term_id FROM user_terms WHERE dictionary_id=? AND kind='override'
            AND source_text=? AND source_locale IS ? AND target_locale=? AND usage_context_id IS ?""",
            (dictionary, source, source_locale, target, usage),
        ).fetchone()[0]

    def put_preference(
        self,
        dictionary,
        source,
        replacement,
        target,
        usage=None,
        source_locale=None,
        note=None,
    ):
        with self.conn:
            return self._put(
                dictionary, source, replacement, target, usage, source_locale, note
            )

    def delete_term(self, id):
        with self.conn:
            self.conn.execute("DELETE FROM user_terms WHERE user_term_id=?", (id,))

    def edit_preference(
        self, id, source, replacement, target, usage=None, source_locale=None, note=None
    ):
        self._validate_term(source, replacement, target, usage, source_locale)
        with self.conn:
            if not self.conn.execute(
                """UPDATE user_terms SET source_text=?,replacement=?,target_locale=?,usage_context_id=?,source_locale=?,note=?,legacy=0,priority=0,
                updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE user_term_id=? AND kind='override'""",
                (source, replacement, target, usage, source_locale, note, id),
            ).rowcount:
                raise ValueError("Unknown preferred term")

    def set_term_enabled(self, id, enabled):
        with self.conn:
            self.conn.execute(
                "UPDATE user_terms SET enabled=? WHERE user_term_id=?",
                (int(enabled), id),
            )

    def preview_csv(self, text, target=None):
        rows, errors = [], []
        scopes = {}
        try:
            reader = csv.DictReader(io.StringIO(text), strict=True)
            allowed = {
                "source_text",
                "replacement",
                "target_locale",
                "usage_context_id",
                "note",
            }
            if (
                not reader.fieldnames
                or len(set(reader.fieldnames)) != len(reader.fieldnames)
                or not {"source_text", "replacement"} <= set(reader.fieldnames)
                or not set(reader.fieldnames) <= allowed
            ):
                raise ValueError("Invalid CSV header")
            for row in reader:
                try:
                    if None in row or any(value is None for value in row.values()):
                        raise ValueError("Wrong column count")
                    item = {
                        "source": row["source_text"],
                        "replacement": row["replacement"],
                        "target": row.get("target_locale") or target,
                        "usage": row.get("usage_context_id") or None,
                        "note": row.get("note") or None,
                    }
                    self._validate_term(
                        item["source"],
                        item["replacement"],
                        item["target"],
                        item["usage"],
                    )
                    rows.append(item)
                    scope = (item["source"], item["target"], item["usage"])
                    if scope in scopes and scopes[scope] != item["replacement"]:
                        rows.pop()
                        raise ValueError("Conflicting duplicate CSV scope")
                    scopes[scope] = item["replacement"]
                except ValueError as error:
                    errors.append({"row": reader.line_num, "reason": str(error)})
        except (ValueError, csv.Error) as error:
            rows = []
            errors.append({"row": reader.reader.line_num or 1, "reason": str(error)})
        return {"rows": rows, "errors": errors}

    def commit_csv(self, dictionary, preview):
        if preview["errors"]:
            raise ValueError("CSV has invalid rows; nothing imported")
        # Revalidate, never trust a caller-mutated preview.
        with self.conn:
            scopes = {}
            for row in preview["rows"]:
                scope = (row["source"], row["target"], row["usage"])
                if scope in scopes and scopes[scope] != row["replacement"]:
                    raise ValueError("Conflicting duplicate CSV scope")
                scopes[scope] = row["replacement"]
                self._put(
                    dictionary,
                    row["source"],
                    row["replacement"],
                    row["target"],
                    row["usage"],
                    None,
                    row["note"],
                )
