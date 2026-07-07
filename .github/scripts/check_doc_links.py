#!/usr/bin/env python3
"""Docs sanity: fail on broken relative markdown links or merge-conflict
markers. Single implementation shared by CI's docs job and `just links` so
the local mirror can't drift from the gate."""

import re
import sys
from pathlib import Path

SKIP_DIRS = {".git", "target", "node_modules", ".build", "DerivedData"}
LINK = re.compile(r"\]\(([^)\s]+)\)")
CONFLICT = re.compile(r"^(<{7} |>{7} )")
EXTERNAL = ("http://", "https://", "mailto:", "tel:")

failures = []
for md in sorted(Path(".").rglob("*.md")):
    if SKIP_DIRS & set(md.parts):
        continue
    text = md.read_text(encoding="utf-8")
    for lineno, line in enumerate(text.splitlines(), start=1):
        if CONFLICT.match(line):
            failures.append(f"{md}:{lineno}: merge-conflict marker")
    for match in LINK.finditer(text):
        target = match.group(1).split("#", 1)[0]
        if not target or target.startswith(EXTERNAL):
            continue
        if not (md.parent / target).exists():
            failures.append(f"{md}: broken link -> {match.group(1)}")

for failure in failures:
    print(failure)
sys.exit(1 if failures else 0)
