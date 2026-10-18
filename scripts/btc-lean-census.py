#!/usr/bin/env python3
"""Tie docs/btc-lean-census.md lines to // btc-lean: comments.

A mapped row cites `path` line N. Line N is the statement. The comment lines
immediately above it name the census ids. This script fails when those disagree.
It does not read the btc-lean submodule (CI does not check that gitlink out).
"""
from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

COMMENT = re.compile(r"^\s*// btc-lean:((?:\s+[A-Z]{3}-\d{3})+)\s*$")
LOC = re.compile(r"^- (production|test): `([^`]+)` line (\d+)\s*$")
IDH = re.compile(r"^### ([A-Z]{3}-\d{3})\s*$")


def parse_census(text: str) -> list[dict]:
    rows = []
    cur = None
    for ln in text.splitlines():
        m = IDH.match(ln)
        if m:
            if cur:
                rows.append(cur)
            cur = {"id": m.group(1), "prod": None, "test": None, "unmapped": False}
            continue
        if cur is None:
            continue
        if ln.startswith("- unmapped:"):
            reason = ln[len("- unmapped:") :].strip()
            if not reason:
                raise SystemExit(f"{cur['id']}: empty unmapped reason")
            cur["unmapped"] = True
        else:
            loc = LOC.match(ln)
            if loc:
                key = "prod" if loc.group(1) == "production" else "test"
                cur[key] = (loc.group(2), int(loc.group(3)))
    if cur:
        rows.append(cur)
    return rows


def comment_sites(root: Path) -> list[tuple[str, int, list[str]]]:
    sites = []
    for path in sorted(root.rglob("*.rs")):
        rel = path.relative_to(root).as_posix()
        if rel.startswith("target/") or "/target/" in rel:
            continue
        lines = path.read_text().splitlines()
        i = 0
        while i < len(lines):
            m = COMMENT.match(lines[i])
            if not m:
                i += 1
                continue
            ids: list[str] = []
            while i < len(lines) and (m := COMMENT.match(lines[i])):
                ids.extend(m.group(1).split())
                i += 1
            if i >= len(lines):
                raise SystemExit(f"{rel}: btc-lean comment has no statement under it")
            if not lines[i].strip() or lines[i].lstrip().startswith("//"):
                raise SystemExit(f"{rel}:{i+1}: btc-lean comment is not on the next code line")
            sites.append((rel, i + 1, ids))
            i += 1
    return sites


def check(root: Path) -> list[str]:
    census = root / "docs" / "btc-lean-census.md"
    rows = parse_census(census.read_text())
    if not rows:
        return ["census has no rows"]
    errs: list[str] = []
    seen = set()
    mapped: dict[str, dict] = {}
    for row in rows:
        if row["id"] in seen:
            errs.append(f"duplicate census id {row['id']}")
        seen.add(row["id"])
        if row["unmapped"]:
            if row["prod"] or row["test"]:
                errs.append(f"{row['id']}: unmapped row also cites a line")
            continue
        if not row["prod"] or not row["test"]:
            errs.append(f"{row['id']}: mapped row needs a production line and a test line")
            continue
        mapped[row["id"]] = row
        for role in ("prod", "test"):
            path, line = row[role]
            file = root / path
            if not file.is_file():
                errs.append(f"{row['id']}: missing {path}")
                continue
            src = file.read_text().splitlines()
            if line < 1 or line > len(src):
                errs.append(f"{row['id']}: {path}:{line} is past EOF")

    sites = comment_sites(root)
    at: dict[str, list[tuple[str, int]]] = {}
    for rel, line, ids in sites:
        dup = [i for i in ids if ids.count(i) > 1]
        for i in sorted(set(dup)):
            errs.append(f"{rel}:{line}: {i} repeated in one comment")
        for i in ids:
            at.setdefault(i, []).append((rel, line))

    for rid, row in mapped.items():
        want = {row["prod"], row["test"]}
        got = set(at.get(rid, []))
        if got != want:
            errs.append(f"{rid}: comment sites {sorted(got)} != census {sorted(want)}")
    for rid, places in at.items():
        if rid not in mapped:
            errs.append(f"{rid}: comment has no mapped census row ({places[0]})")
    return errs


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs").mkdir()
        (root / "src").mkdir()
        (root / "src" / "a.rs").write_text(
            "// btc-lean: INV-001\n"
            "fn prod() {}\n"
            "// btc-lean: INV-001\n"
            "fn test_it() {}\n"
        )
        (root / "docs" / "btc-lean-census.md").write_text(
            "### INV-001\n"
            "- summary: empty vin\n"
            "- census: proven consensus\n"
            "- production: `src/a.rs` line 2\n"
            "- test: `src/a.rs` line 4\n"
            "### INV-002\n"
            "- summary: retired\n"
            "- census: obsolete\n"
            "- unmapped: retired in the vendored census\n"
        )
        errs = check(root)
        if errs:
            raise SystemExit("self-test clean tree failed:\n" + "\n".join(errs))
        text = (root / "docs" / "btc-lean-census.md").read_text().replace("line 2", "line 3")
        (root / "docs" / "btc-lean-census.md").write_text(text)
        errs = check(root)
        if not any("INV-001" in e for e in errs):
            raise SystemExit("self-test did not notice a drifted line")
    print("btc-lean-census self-test ok")


def main() -> None:
    if "--self-test" in sys.argv:
        self_test()
        return
    root = Path.cwd()
    if "--root" in sys.argv:
        root = Path(sys.argv[sys.argv.index("--root") + 1])
    errs = check(root)
    if errs:
        print(f"btc-lean census: {len(errs)} error(s)", file=sys.stderr)
        for e in errs[:80]:
            print(e, file=sys.stderr)
        raise SystemExit(1)
    print(f"btc-lean census: comments match docs/btc-lean-census.md")


if __name__ == "__main__":
    main()
