#!/usr/bin/env python3
"""Manage the local user dictionary database."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from user_dictionary import UserDictionary  # noqa: E402


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument(
        "--db",
        type=Path,
        default=Path.home() / ".chinese-regional-localizer" / "user_dictionary.sqlite",
    )
    root.add_argument(
        "--schema",
        type=Path,
        default=REPO_ROOT / "schema" / "user-dictionary-v0.1.sql",
    )
    commands = root.add_subparsers(dest="command", required=True)

    protected = commands.add_parser("protect", help="Protect a term from all conversion")
    protected.add_argument("text")
    _scope_args(protected)
    protected.add_argument("--priority", type=int, default=0)
    protected.add_argument("--note")

    override = commands.add_parser("override", help="Add or replace a fixed user translation")
    override.add_argument("text")
    override.add_argument("replacement")
    _scope_args(override)
    override.add_argument("--priority", type=int, default=0)
    override.add_argument("--note")

    commands.add_parser("list", help="List user terms")

    remove = commands.add_parser("remove", help="Remove a user term by ID")
    remove.add_argument("id", type=int)

    enable = commands.add_parser("enable", help="Enable a user term by ID")
    enable.add_argument("id", type=int)

    disable = commands.add_parser("disable", help="Disable a user term by ID")
    disable.add_argument("id", type=int)
    return root


def _scope_args(command: argparse.ArgumentParser) -> None:
    command.add_argument("--source-locale")
    command.add_argument("--target-locale")


def main() -> int:
    args = parser().parse_args()
    dictionary = UserDictionary.open(args.db, args.schema)
    try:
        if args.command == "protect":
            term_id = dictionary.add_protected(
                args.text,
                source_locale=args.source_locale,
                target_locale=args.target_locale,
                priority=args.priority,
                note=args.note,
            )
            print(f"protected term #{term_id}")
        elif args.command == "override":
            term_id = dictionary.add_override(
                args.text,
                args.replacement,
                source_locale=args.source_locale,
                target_locale=args.target_locale,
                priority=args.priority,
                note=args.note,
            )
            print(f"override term #{term_id}")
        elif args.command == "list":
            for term in dictionary.list_terms():
                scope = f"{term.source_locale or '*'} -> {term.target_locale or '*'}"
                replacement = term.replacement if term.replacement is not None else "[protected]"
                state = "on" if term.enabled else "off"
                print(
                    f"{term.user_term_id}\t{state}\t{term.kind}\t{scope}\t"
                    f"{term.source_text}\t{replacement}\tpriority={term.priority}"
                )
        elif args.command == "remove":
            if not dictionary.remove(args.id):
                print(f"term #{args.id} not found", file=sys.stderr)
                return 1
        elif args.command in {"enable", "disable"}:
            if not dictionary.set_enabled(args.id, args.command == "enable"):
                print(f"term #{args.id} not found", file=sys.stderr)
                return 1
    finally:
        dictionary.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
