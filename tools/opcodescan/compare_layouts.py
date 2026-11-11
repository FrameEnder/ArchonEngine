#!/usr/bin/env python3
"""Checks a new game version's opcodes against the last one's by how the client's code reads each packet.

    python3 tools/opcodescan/compare_layouts.py <old layouts.json> <new layouts.json>

Make each layouts.json with `kawari-opcodescan <exe> layouts <opcodes.yml> <fork_opcodes.yml> <version> core/src <out>`,
the old one from the old executable with the old version's opcode files (e.g. `git show <commit>:resources/data/...`).
A packet's handler reads the same way from one patch to the next unless its layout changed, so a packet whose
`extent` (how far into the packet its handler reads) differs is either on the wrong opcode or really changed, and
needs a look. On 7.58 (2026.11.03) every one of 344 packets read as on 7.57h1 except two: upstream's EditSearchInfo,
still on 7.57h1's opcode, and PingSync, unmapped on 7.57h1; the fingerprint matcher's 24 wrong server opcodes would
all have shown here (their extents were those of other packets.)

Only numbers are read and printed: packet names, opcodes and extents.
"""

import json
import sys


def packets(path):
    with open(path) as f:
        return {(p["kind"], p["name"]): p for p in json.load(f)["packets"]}


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    old, new = packets(sys.argv[1]), packets(sys.argv[2])
    differ = 0
    for key in sorted(set(old) | set(new)):
        a, b = old.get(key, {}), new.get(key, {})
        if a.get("extent") != b.get("extent"):
            differ += 1
            print(
                f"{key[0]:6} {key[1]:32} old opcode {a.get('opcode')} reads {a.get('extent')}"
                f"   new opcode {b.get('opcode')} reads {b.get('extent')}"
            )
    print(f"{differ} of {len(set(old) | set(new))} packets read differently")
    return 1 if differ else 0


if __name__ == "__main__":
    sys.exit(main())
