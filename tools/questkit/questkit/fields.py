"""What retail sends that Kawari doesn't: packets it never sends, and bytes of shared packets retail fills in that
Kawari always leaves zero (`questkit fields`).

Retail's side is Chronofoil captures (only those of the game version Kawari's opcodes are for, unless asked: older
layouts differ), Kawari's its own packet captures (diagnostics bundles' `captures/`). Only server packets count.
Nothing of the game is kept: the report has packet names, offsets, sizes and how often each byte was set, with a few
of retail's values for each run of bytes (numbers, as everything else from captures.)
"""

from __future__ import annotations

import struct
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path

# Packets that carry other players (names, content IDs, their gear): left out, since captures hold other people's
# data and only numbers of one's own may come out of them.
OTHER_PLAYERS = {
    "Blacklist", "SocialList", "UnkSocialResponse", "FriendList", "Linkshells", "CrossworldLinkshells",
    "CrossworldLinkshellMembers", "LinkshellMembers", "FreeCompanyHierarchy", "FreeCompanyGreeting",
    "FreeCompanyInfo", "CharaInfoFromContentIds", "ExamineCharacterInformation", "PartyList", "PartyUpdate",
    "SpawnPlayer", "PlayerName", "SearchPlayersResult", "CrossRealmListingsOverview", "CrossRealmListings",
    "CrossRealmListingInformation", "MarketBoardSearchResult", "MarketBoardItemListing",
    "MarketBoardItemListingHistory", "Mail", "Letter", "LetterList", "TellNotFound", "InviteReply",
    "PartyInvite", "RetainerInformation", "PartyMemberPositions",
}

# A run of bytes is reported when retail set it in at least this share of its packets (and Kawari never did.)
DEFAULT_SHARE = 0.1
# Packets seen fewer times than this on retail aren't reported (too few to tell what's usual.)
DEFAULT_MIN_COUNT = 3


@dataclass
class Stats:
    count: int = 0
    sizes: Counter = field(default_factory=Counter)
    set_bytes: Counter = field(default_factory=Counter)  # offset -> packets with that byte non-zero
    samples: list = field(default_factory=list)  # a few packets' data, to show values from

    def add(self, data: bytes) -> None:
        self.count += 1
        self.sizes[len(data)] += 1
        for offset, byte in enumerate(data):
            if byte:
                self.set_bytes[offset] += 1
        if len(self.samples) < 200:
            self.samples.append(data)


def retail_stats(capture_paths, all_versions: bool = False) -> tuple[dict[str, Stats], list[str]]:
    from . import chronofoil

    tables = chronofoil.load_opcodes()
    current, _ = chronofoil._current_opcodes()
    stats: dict[str, Stats] = defaultdict(Stats)
    used = []
    for capture in chronofoil.find_captures(capture_paths):
        if not all_versions and capture.game_version != current:
            continue
        used.append(f"{capture.id[:8]} ({capture.game_version})")
        names = chronofoil.Names(tables, capture.game_version)
        for segment in capture.segments(protocols=("zone",)):
            if segment.dir != "out":
                continue
            name = names.name(segment)
            if name in chronofoil.PRIVATE or name in OTHER_PLAYERS:
                continue
            stats[name].add(segment.data)
    return stats, used


def kawari_stats(capture_paths) -> tuple[dict[str, Stats], int]:
    from . import capture as kawari_capture

    stats: dict[str, Stats] = defaultdict(Stats)
    files = 0
    for path in capture_paths:
        path = Path(path)
        found = sorted(path.rglob("*.jsonl*")) if path.is_dir() else [path]
        for file in found:
            files += 1
            for packet in kawari_capture.load(file):
                if packet.dir != "out" or not packet.hex:
                    continue
                stats[packet.name].add(packet.data)
    return stats, files


def _runs(offsets: list[int]) -> list[tuple[int, int]]:
    """Consecutive offsets as (start, end) inclusive."""
    runs = []
    for offset in sorted(offsets):
        if runs and offset == runs[-1][1] + 1:
            runs[-1] = (runs[-1][0], offset)
        else:
            runs.append((offset, offset))
    return runs


def _values(samples: list[bytes], start: int, end: int, limit: int = 4) -> str:
    """Retail's commonest values for bytes start..end, read as the widest little-endian integer that fits."""
    width = end - start + 1
    fmt = {1: "<B", 2: "<H", 4: "<I", 8: "<Q"}.get(width)
    seen = Counter()
    for data in samples:
        if len(data) <= end:
            continue
        chunk = data[start:end + 1]
        if not any(chunk):
            continue
        seen[struct.unpack(fmt, chunk)[0] if fmt else chunk.hex()] += 1
    return ", ".join(f"{value}" + (f" ×{n}" if n > 1 else "") for value, n in seen.most_common(limit))


def report(retail: dict[str, Stats], kawari: dict[str, Stats], share: float = DEFAULT_SHARE,
           min_count: int = DEFAULT_MIN_COUNT, only: set[str] | None = None) -> str:
    lines = []
    if only:
        retail = {name: stats for name, stats in retail.items() if name in only}
    never = sorted((name for name, stats in retail.items() if stats.count >= min_count and name not in kawari),
                   key=lambda name: -retail[name].count)
    lines.append("## Packets retail sends that Kawari never sent")
    lines.append("")
    lines.append("| Packet | Times on retail | Size | Bytes ever set |")
    lines.append("|---|---|---|---|")
    for name in never:
        stats = retail[name]
        size = stats.sizes.most_common(1)[0][0]
        lines.append(f"| {name} | {stats.count} | {size} | {len(stats.set_bytes)} |")
    lines.append("")

    lines.append("## Bytes retail fills in that Kawari always sent as zero")
    lines.append("")
    lines.append(f"(Set in at least {share:.0%} of retail's packets of the kind; offsets are into the IPC data.)")
    lines.append("")
    lines.append("| Packet | Retail / Kawari packets | Bytes | Retail set it | Retail's values |")
    lines.append("|---|---|---|---|---|")
    for name in sorted(retail):
        stats = retail[name]
        if stats.count < min_count or name not in kawari:
            continue
        ours = kawari[name]
        offsets = [offset for offset, n in stats.set_bytes.items()
                   if n >= share * stats.count and ours.set_bytes.get(offset, 0) == 0]
        for start, end in _runs(offsets):
            # Integers are aligned: split a run into its natural pieces so values read right.
            pieces = []
            at = start
            while at <= end:
                for width in (8, 4, 2, 1):
                    if at % width == 0 and at + width - 1 <= end:
                        pieces.append((at, at + width - 1))
                        at += width
                        break
            for piece_start, piece_end in pieces:
                most = max(stats.set_bytes[o] for o in range(piece_start, piece_end + 1))
                span = f"{piece_start}" if piece_start == piece_end else f"{piece_start}–{piece_end}"
                lines.append(f"| {name} | {stats.count} / {ours.count} | {span} | {most / stats.count:.0%} | "
                             f"{_values(stats.samples, piece_start, piece_end)} |")
    lines.append("")

    lines.append("## Packet sizes that differ")
    lines.append("")
    for name in sorted(retail):
        if name not in kawari or retail[name].count < min_count:
            continue
        theirs = retail[name].sizes.most_common(1)[0][0]
        ours = kawari[name].sizes.most_common(1)[0][0]
        if theirs != ours:
            lines.append(f"* {name}: retail {theirs} bytes, Kawari {ours}")
    lines.append("")
    return "\n".join(lines)
