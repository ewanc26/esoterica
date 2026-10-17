#!/usr/bin/env python3
"""Generate data/phonologies_glottolog.toml: one representative inventory per
Glottolog top-level family or isolate, excluding sign languages.

Sources (download first, see README "Regenerating the generated catalogue"):
  * PHOIBLE segment inventories: phoible.csv from https://github.com/phoible/dev
    (data/phoible.csv, CC BY-SA 3.0).
  * Glottolog repository archive: https://github.com/glottolog/glottolog
    (languoids/tree/<glottocode>/md.ini gives each languoid's name, level and
    place in the family tree).

Method:
  * A family's members are every languoid in its subtree.
  * Each language with a PHOIBLE inventory uses the inventory with the most
    non-marginal segments.
  * The representative is the member whose consonant count is the median
    (ties broken by glottocode), so an extreme inventory is not chosen.
  * Consonants, vowels and tones come from PHOIBLE's SegmentClass column.
  * The syllable template is a generic default, "CV(C)". PHOIBLE does not record
    phonotactics, so the template is not sourced from the language.

Families with no PHOIBLE inventory in their subtree are not emitted. They are
listed in the output's header comment, so the gap is visible.

Usage:
  python3 scripts/build_glottolog_phonologies.py \
      --phoible path/to/phoible.csv --glottolog path/to/glottolog-master \
      --out data/phonologies_glottolog.toml
"""
import argparse
import csv
import json
import os
import re
import statistics
import sys
from collections import defaultdict

SIGN_FAMILY = "sign1238"
DEFAULT_TEMPLATE = "CV(C)"
MAX_TONES = 9


def read_languoids(tree):
    """Return {glottocode: (name, level, children)} for every languoid under tree/."""
    info = {}
    children = defaultdict(list)

    def walk(path, parent):
        for entry in sorted(os.listdir(path)):
            sub = os.path.join(path, entry)
            ini = os.path.join(sub, "md.ini")
            if not os.path.isdir(sub) or not os.path.exists(ini):
                continue
            text = open(ini, encoding="utf-8").read()
            name = re.search(r"^name = (.*)$", text, re.M)
            level = re.search(r"^level = (.*)$", text, re.M)
            info[entry] = (name.group(1).strip() if name else entry,
                           level.group(1).strip() if level else "")
            if parent:
                children[parent].append(entry)
            walk(sub, entry)

    walk(tree, None)
    return info, children


def subtree(root, children):
    out, stack = set(), [root]
    while stack:
        g = stack.pop()
        if g in out:
            continue
        out.add(g)
        stack.extend(children.get(g, []))
    return out


def read_inventories(phoible_csv):
    """Return {glottocode: {InventoryID: {'name', 'segments': [(phoneme, cls, marginal)]}}}."""
    inv = defaultdict(lambda: defaultdict(lambda: {"name": "", "segments": []}))
    with open(phoible_csv, encoding="utf-8", newline="") as fh:
        for row in csv.DictReader(fh):
            gc = row["Glottocode"]
            if not gc:
                continue
            entry = inv[gc][row["InventoryID"]]
            entry["name"] = row["LanguageName"]
            entry["segments"].append(
                (row["Phoneme"], row["SegmentClass"], row["Marginal"].strip().lower() == "true"))
    return inv


def best_inventory(by_id):
    """The inventory with the most non-marginal segments."""
    def score(item):
        inv_id, data = item
        return (sum(1 for _, _, m in data["segments"] if not m), inv_id)
    inv_id, data = max(by_id.items(), key=score)
    return inv_id, data


def build_entry(inv_id, data, cls_wanted):
    segs = data["segments"]
    core = [s for s in segs if not s[2]] or segs
    def pick(cls):
        return sorted({p for p, c, _ in core if c == cls and p})
    return {
        "inventory": inv_id,
        "name": data["name"],
        "consonants": pick("consonant"),
        "vowels": pick("vowel"),
        "tones": len(pick("tone")),
    }


def toml_str(s):
    return json.dumps(s, ensure_ascii=False)


def toml_list(items):
    return "[" + ", ".join(toml_str(i) for i in items) + "]"


def slug(name):
    s = re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")
    return s or "family"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--phoible", required=True)
    ap.add_argument("--glottolog", required=True, help="glottolog-master directory")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    tree = os.path.join(args.glottolog, "languoids", "tree")
    info, children = read_languoids(tree)
    inventories = read_inventories(args.phoible)
    best = {gc: best_inventory(by_id) for gc, by_id in inventories.items()}

    top_level = sorted(g for g in os.listdir(tree)
                       if os.path.isdir(os.path.join(tree, g)) and g in info)

    entries, unsourced = [], []
    for group in top_level:
        if group == SIGN_FAMILY:
            continue
        name, level = info[group]
        members = subtree(group, children)
        candidates = []
        for gc in sorted(members):
            if gc in best:
                inv_id, data = best[gc]
                candidates.append((gc, inv_id, data))
        candidates = [c for c in candidates if sum(1 for _, cls, _ in c[2]["segments"] if cls == "consonant") > 0]
        if not candidates:
            unsourced.append((group, name))
            continue
        candidates.sort(key=lambda c: (sum(1 for _, cls, _ in c[2]["segments"] if cls == "consonant"), c[0]))
        counts = [sum(1 for _, cls, _ in c[2]["segments"] if cls == "consonant") for c in candidates]
        target = statistics.median_low(counts)
        chosen = next(c for c, n in zip(candidates, counts) if n == target)
        gc, inv_id, data = chosen
        entry = build_entry(inv_id, data, None)
        if not entry["vowels"] or not entry["consonants"]:
            unsourced.append((group, name))
            continue
        entries.append((group, name, gc, entry))

    lines = [
        "# Generated by scripts/build_glottolog_phonologies.py. Do not edit by hand.",
        "# One representative PHOIBLE inventory per Glottolog top-level family or isolate,",
        "# excluding sign languages. The syllable template is a generic default (\"CV(C)\"),",
        "# not sourced from the language. See the script header for the method.",
        "#",
        f"# {len(entries)} entries; {len(unsourced)} families or isolates had no usable PHOIBLE inventory",
        "# and are not emitted:",
    ]
    for group, name in unsourced:
        lines.append(f"#   {group} {name}")
    lines.append("")

    for group, name, gc, entry in entries:
        key = f"{slug(name)}_{group}"
        lines.append(f"[{key}]")
        lines.append(f"# Glottolog {group} ({name}). PHOIBLE inventory {entry['inventory']} "
                     f"for {entry['name']} ({gc}).")
        lines.append(f"vowels = {toml_list(entry['vowels'])}")
        lines.append(f"consonants = {toml_list(entry['consonants'])}")
        lines.append(f"syllable_structure = {toml_str(DEFAULT_TEMPLATE)}")
        if entry["tones"] >= 2:
            lines.append(f"tones = {min(entry['tones'], MAX_TONES)}")
        lines.append("vowel_harmony = false")
        lines.append("")

    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines))

    print(f"emitted {len(entries)} entries; {len(unsourced)} unsourced", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
