#!/usr/bin/env python3
"""Convert the app's SQLite dump (backup.sql) into a PostgreSQL dump (pgdump.sql).

The SQLite dump is emitted with SeaORM's portable text-backed types
(uuid_text, timestamp_with_timezone_text, json_text, enum_text, date_text),
16-byte hex blobs (X'...'), and unistr('...') escapes. This script rewrites
it into native PostgreSQL dialect:

  uuid_text                      -> UUID          (X'...' -> 'xxxx-...' literal)
  timestamp_with_timezone_text   -> TIMESTAMPTZ
  json_text                      -> JSONB
  enum_text                      -> native ENUM type (tag/priority/notification_type/item_type)
  date_text                      -> DATE
  boolean                        -> BOOLEAN
  varchar / text                 -> TEXT
  integer                        -> BIGINT

unistr('...') bodies are decoded to plain string literals because PostgreSQL's
unistr() rejects \n / \" escapes. Plain '...' strings are preserved verbatim
(backslashes are data, e.g. JSON payloads).

Usage: python3 scripts/sqlite_to_pg.py [--src backup.sql] [--out pgdump.sql]
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ENUM_COLUMNS = {
    ("bookmark", "tag"): "tag",
    ("todo", "priority"): "priority",
    ("notifications", "notification_type"): "notification_type",
    ("recycle_bin", "item_type"): "item_type",
}

ENUM_VALUES = {
    "tag": ["development", "inspiration", "design", "research"],
    "priority": ["high", "medium", "low"],
    "notification_type": [
        "backup_failed",
        "backup_success",
        "workspace_invite_received",
        "workspace_invite_accepted",
        "workspace_invite_declined",
        "item_shared",
        "item_unshared",
        "item_updated",
        "item_deleted",
        "item_access_granted",
        "item_access_revoked",
        "generic",
    ],
    "item_type": ["todo", "note", "reminder", "snippet", "bookmark", "workspace"],
}

TYPE_MAP = {
    "uuid_text": "UUID",
    "varchar": "TEXT",
    "text": "TEXT",
    "integer": "BIGINT",
    "boolean": "BOOLEAN",
    "BOOLEAN": "BOOLEAN",
    "TEXT": "TEXT",
    "timestamp_with_timezone_text": "TIMESTAMPTZ",
    "json_text": "JSONB",
    "enum_text": "enum_text",  # resolved per column via ENUM_COLUMNS
    "date_text": "DATE",
}

# Tables with sync triggers in the dump: (table, record_identifier expr on NEW/OLD)
TRIGGER_TABLES = [
    "bookmark",
    "notes",
    "recycle_bin",
    "reminder",
    "snippets",
    "todo",
    "workspaces",
]

EXPECTED_COUNTS = {
    "seaql_migrations": 32,
    "sync_queue": 197,
    "note_categories": 0,
    "workspaces": 14,
    "snippets": 13,
    "todo": 21,
    "bookmark": 33,
    "reminder": 4,
    "recycle_bin": 80,
    "notes": 94,
    "workspace_preferences": 11,
    "user_preferences": 0,
    "notifications": 643,
}


class ParseError(Exception):
    pass


def split_top_level(s):
    """Split on commas that are not inside parentheses."""
    parts = []
    depth = 0
    cur = []
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


def parse_create_table(line):
    m = re.match(r'CREATE TABLE IF NOT EXISTS "([a-z_]+)"\s*\((.*)\)\s*;?\s*$', line)
    if not m:
        raise ParseError("unexpected CREATE TABLE: " + line[:80])
    table = m.group(1)
    body = m.group(2)
    columns = []
    fks = []
    for raw_part in split_top_level(body):
        part = raw_part.strip()
        if not part:
            continue
        if part.startswith("FOREIGN KEY"):
            fks.append(part)
            continue
        cm = re.match(r'"?([a-z_][a-z0-9_]*)"?\s+([a-zA-Z_]+)(.*)$', part)
        if not cm:
            raise ParseError(f"cannot parse column in {table}: {part[:80]}")
        name, sqlite_type, constraints = cm.group(1), cm.group(2), cm.group(3)
        pg_type = TYPE_MAP.get(sqlite_type)
        if pg_type is None:
            raise ParseError(f"unknown type {sqlite_type!r} for {table}.{name}")
        if pg_type == "enum_text":
            enum = ENUM_COLUMNS.get((table, name))
            if enum is None:
                raise ParseError(f"no enum mapping for {table}.{name}")
            pg_type = enum
        columns.append((name, pg_type, constraints.strip()))
    return table, columns, fks


def read_string(s, start):
    """Read a SQL single-quoted string from s[start:] (s[start] == \"'\")."""
    i = start + 1
    out = []
    n = len(s)
    while True:
        if i >= n:
            raise ParseError("unterminated string literal")
        c = s[i]
        if c == "'":
            if i + 1 < n and s[i + 1] == "'":
                out.append("'")
                i += 2
                continue
            return "".join(out), i + 1
        out.append(c)
        i += 1


def decode_unistr(s):
    """Decode the unistr('...') escape repertoire used by the dump."""
    out = []
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        nxt = s[i + 1] if i + 1 < n else ""
        if nxt == "u":
            h = s[i + 2 : i + 6]
            if len(h) != 4 or not re.fullmatch(r"[0-9a-fA-F]{4}", h):
                raise ParseError(f"bad \\u escape: {s[i:i+6]!r}")
            out.append(chr(int(h, 16)))
            i += 6
        elif nxt == "\\":
            out.append("\\")
            i += 2
        elif nxt == "n":
            out.append("\n")
            i += 2
        elif nxt == '"':
            out.append('"')
            i += 2
        else:
            raise ParseError(f"unhandled backslash escape: {s[i:i+4]!r}")
    return "".join(out)


def quote_literal(value):
    return "'" + value.replace("'", "''") + "'"


def uuid_literal(hexstr):
    if len(hexstr) != 32:
        raise ParseError(f"expected 16-byte blob, got {len(hexstr)//2} bytes: X'{hexstr}'")
    return "'{}-{}-{}-{}-{}'".format(
        hexstr[0:8], hexstr[8:12], hexstr[12:16], hexstr[16:20], hexstr[20:32]
    )


def parse_values(inner):
    tokens = []
    i = 0
    n = len(inner)
    while i < n:
        c = inner[i]
        if c in " \t":
            i += 1
            continue
        if c == ",":
            i += 1
            continue
        if inner.startswith("X'", i) or inner.startswith("x'", i):
            j = inner.find("'", i + 2)
            if j == -1:
                raise ParseError("unterminated blob literal")
            tokens.append(("blob", inner[i + 2 : j]))
            i = j + 1
            continue
        if inner.startswith("unistr(", i):
            k = i + len("unistr(")
            while k < n and inner[k] in " \t":
                k += 1
            if k >= n or inner[k] != "'":
                raise ParseError("malformed unistr( call")
            raw, k = read_string(inner, k)
            if k >= n or inner[k] != ")":
                raise ParseError("malformed unistr( call (missing ')')")
            tokens.append(("str", decode_unistr(raw)))
            i = k + 1
            continue
        if c == "'":
            val, j = read_string(inner, i)
            tokens.append(("str", val))
            i = j
            continue
        j = i
        while j < n and inner[j] != ",":
            j += 1
        tokens.append(("raw", inner[i:j].strip()))
        i = j
    return tokens


def convert_insert(line, table, columns):
    m = re.match(r"INSERT INTO ([a-z_]+) VALUES\((.*)\)\s*;\s*$", line)
    if not m:
        raise ParseError("unexpected INSERT: " + line[:80])
    tokens = parse_values(m.group(2))
    if len(tokens) != len(columns):
        raise ParseError(
            f"{table}: expected {len(columns)} values, got {len(tokens)}"
        )
    out = []
    for (kind, val), (name, pg_type, _constraints) in zip(tokens, columns):
        if pg_type == "UUID":
            if kind == "raw" and val == "NULL":
                out.append("NULL")
            elif kind != "blob":
                raise ParseError(f"{table}.{name}: expected uuid blob, got {val!r}")
            else:
                out.append(uuid_literal(val))
        elif pg_type == "BOOLEAN":
            if kind == "raw" and val in ("0", "1"):
                out.append("TRUE" if val == "1" else "FALSE")
            elif kind == "raw" and val.upper() in ("TRUE", "FALSE", "NULL"):
                out.append(val.upper())
            else:
                raise ParseError(f"{table}.{name}: bad boolean {val!r}")
        elif pg_type == "JSONB":
            if kind == "raw" and val == "NULL":
                out.append("NULL")
            else:
                if kind != "str":
                    raise ParseError(f"{table}.{name}: expected json string, got {val!r}")
                json.loads(val)  # validate
                out.append(quote_literal(val))
        elif pg_type in ENUM_VALUES:
            if kind == "raw" and val == "NULL":
                out.append("NULL")
            elif kind == "str" and val in ENUM_VALUES[pg_type]:
                out.append(quote_literal(val))
            else:
                raise ParseError(
                    f"{table}.{name}: value {val!r} not in enum {pg_type}"
                )
        elif kind == "str":
            out.append(quote_literal(val))
        elif kind == "raw":
            out.append(val)
        else:
            raise ParseError(f"{table}.{name}: unexpected blob token")
    return "INSERT INTO {} VALUES({});".format(table, ", ".join(out))


def make_trigger_ddl(table):
    statements = []
    for op in ("insert", "update", "delete"):
        rec = "NEW" if op != "delete" else "OLD"
        fn = f"{table}_sync_{op}_fn"
        statements.append(
            f"CREATE FUNCTION {fn}() RETURNS trigger AS $fn$\n"
            f"BEGIN\n"
            f"  INSERT INTO sync_queue(identifier, table_name, record_identifier, operation, created_at)\n"
            f"  VALUES (gen_random_uuid(), '{table}', replace({rec}.identifier::text, '-', ''), '{op.upper()}', to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH:MM:SS'));\n"
            f"  RETURN {rec};\n"
            f"END;\n"
            f"$fn$ LANGUAGE plpgsql;"
        )
        statements.append(
            f"CREATE TRIGGER {table}_sync_{op}\n"
            f"AFTER {op.upper()} ON {table}\n"
            f"FOR EACH ROW\n"
            f"EXECUTE FUNCTION {fn}();"
        )
    return statements


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=str(ROOT / "backup.sql"))
    ap.add_argument("--out", default=str(ROOT / "pgdump.sql"))
    args = ap.parse_args()

    src = Path(args.src)
    out = Path(args.out)
    lines = src.read_text(encoding="utf-8").split("\n")

    tables = {}
    order = []
    schema_lines = []
    data_lines = []
    trigger_stmts = []
    counts = {t: 0 for t in EXPECTED_COUNTS}
    in_trigger = False

    for raw in lines:
        line = raw.rstrip("\n")
        stripped = line.strip()
        if not stripped:
            continue
        if stripped == "PRAGMA foreign_keys=OFF;":
            continue
        if stripped == "BEGIN TRANSACTION;":
            continue
        if stripped == "COMMIT;":
            continue
        if stripped.startswith("CREATE TRIGGER"):
            # Reconstructed separately below; skip the SQLite bodies (which end
            # with a bare "END;" line).
            in_trigger = True
            continue
        if in_trigger:
            if stripped == "END;":
                in_trigger = False
            continue
        if stripped.startswith("CREATE TABLE"):
            table, columns, fks = parse_create_table(stripped)
            if table in tables:
                raise ParseError(f"duplicate CREATE TABLE {table}")
            tables[table] = (columns, fks)
            order.append(table)
            schema_lines.append(
                render_create_table(table, columns, fks)
            )
            continue
        if stripped.startswith("INSERT INTO"):
            m = re.match(r"INSERT INTO ([a-z_]+) VALUES", stripped)
            if m is None:
                raise ParseError("unexpected INSERT: " + line[:80])
            table = m.group(1)
            if table not in tables:
                raise ParseError(f"INSERT into unknown table {table}")
            converted = convert_insert(stripped, table, tables[table][0])
            data_lines.append(converted)
            counts[table] = counts.get(table, 0) + 1
            continue
        raise ParseError(f"unhandled line: {line[:80]}")

    if order != list(EXPECTED_COUNTS):
        raise ParseError(f"unexpected table order: {order}")

    for table, expected in EXPECTED_COUNTS.items():
        if counts[table] != expected:
            raise ParseError(
                f"count mismatch for {table}: expected {expected}, got {counts[table]}"
            )

    for table in TRIGGER_TABLES:
        trigger_stmts.extend(make_trigger_ddl(table))

    out_lines = []
    out_lines.append("-- PostgreSQL dump generated from backup.sql (SQLite)")
    out_lines.append(f"-- by scripts/sqlite_to_pg.py")
    out_lines.append("BEGIN;")
    out_lines.append("")
    out_lines.append("SET standard_conforming_strings = on;")
    out_lines.append("")
    out_lines.append("-- Enum types (match the app's SeaORM migrations)")
    for enum, values in ENUM_VALUES.items():
        vals = ", ".join("'{}'".format(v) for v in values)
        out_lines.append(f"CREATE TYPE {enum} AS ENUM ({vals});")
    out_lines.append("")
    out_lines.append("-- Tables")
    out_lines.extend(schema_lines)
    out_lines.append("")
    out_lines.append("-- Data")
    out_lines.extend(data_lines)
    out_lines.append("")
    out_lines.append("-- Sync triggers (converted to PL/pgSQL)")
    out_lines.extend(trigger_stmts)
    out_lines.append("")
    out_lines.append("COMMIT;")

    out.write_text("\n".join(out_lines) + "\n", encoding="utf-8")
    print(f"wrote {out} ({len(out_lines)} lines)")
    for table in order:
        print(f"  {table:24} {counts[table]} rows")


def render_create_table(table, columns, fks):
    cols = ['  "{}" {} {}'.format(name, pg_type, constraints).rstrip() for name, pg_type, constraints in columns]
    for fk in fks:
        cols.append("  " + fk)
    body = ",\n".join(cols)
    return f"CREATE TABLE IF NOT EXISTS {table} (\n{body}\n);"


if __name__ == "__main__":
    try:
        main()
    except ParseError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
