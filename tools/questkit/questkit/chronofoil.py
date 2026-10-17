"""Reading Chronofoil captures (retail packet captures, https://github.com/ProjectChronofoil/Chronofoil.Plugin).

A capture is one session, from connecting to a data centre to logging out:

* finished: a zip (`<id>.cfcap`, or `<id>.ccfcap` once censored) of three parts, `VersionInfo` and `CaptureInfo`
  (length-delimited protobuf messages: the game's versions, the capture's ID, start and end) and `Data` (zstd: the
  frames, each a length-delimited protobuf `CaptureFrame` of protocol, direction and the game's frame as it arrived,
  already decompressed);
* unfinished (the game closed without logging out): a folder `<id>/` with the same three files, `Data` not compressed.

The format is Chronofoil.CaptureFile's (`src/Proto/ChronofoilCaptureFile.proto`, `src/CaptureReader.cs`). A game frame
is a 40-byte header (time in ms at 16, packet count at 30), then packets: a 16-byte element header (size, source and
target actor, type at 12; type 3 is IPC), then, for IPC, the 16-byte IPC header (opcode at 2) and the data.

Opcodes change with every game version, so each version needs its own table: see `data/chronofoil_opcodes.json` and
`load_opcodes`. Captures hold everything the game sent, other players' characters and (uncensored) chat included, so
they never go in git; exports leave out chat.
"""

from __future__ import annotations

import gzip
import io
import json
import struct
import zipfile
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

OPCODES_FILE = Path(__file__).parent / "data" / "chronofoil_opcodes.json"
REPO = Path(__file__).resolve().parents[3]

PROTOCOLS = {1: "zone", 2: "chat", 3: "lobby"}
KINDS = {
    ("zone", "out"): "ServerZoneIpcType",
    ("zone", "in"): "ClientZoneIpcType",
    ("chat", "out"): "ServerChatIpcType",
    ("chat", "in"): "ClientChatIpcType",
}
# Packets left out of exports: what players said and wrote.
PRIVATE = {"ChatMessage", "SendChatMessage", "TellMessage", "PartyMessage", "LinkshellMessage", "FreeCompanyMessage"}
# Older names for packets Kawari has since renamed.
ALIASES = {"ActionResult": "ActionEffect1"}


# --- protobuf, as much as the capture format needs ---

def _varint(data: bytes, i: int) -> tuple[int, int]:
    value = shift = 0
    while True:
        byte = data[i]
        i += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            return value, i


def _fields(data: bytes) -> dict[int, list]:
    out: dict[int, list] = defaultdict(list)
    i = 0
    while i < len(data):
        key, i = _varint(data, i)
        number, wire = key >> 3, key & 7
        if wire == 0:
            value, i = _varint(data, i)
        elif wire == 2:
            length, i = _varint(data, i)
            value = data[i:i + length]
            i += length
        elif wire == 1:
            value, i = data[i:i + 8], i + 8
        elif wire == 5:
            value, i = data[i:i + 4], i + 4
        else:
            raise ValueError(f"unsupported protobuf wire type {wire}")
        out[number].append(value)
    return out


def _delimited(stream):
    """Length-delimited messages from a stream, until it ends (or a message is cut short)."""
    while True:
        length = shift = 0
        while True:
            byte = stream.read(1)
            if not byte:
                return
            length |= (byte[0] & 0x7F) << shift
            shift += 7
            if byte[0] < 0x80:
                break
        message = stream.read(length)
        if len(message) < length:
            return
        yield message


def _timestamp(message: bytes | None) -> datetime | None:
    if not message:
        return None
    seconds = _fields(message).get(1, [0])[0]
    return datetime.fromtimestamp(seconds, timezone.utc) if seconds > 0 else None


def _zstd_reader(stream):
    try:
        from compression import zstd  # Python 3.14

        return zstd.ZstdFile(stream)
    except ImportError:
        pass
    try:
        import zstandard
    except ImportError:
        raise SystemExit("Reading finished Chronofoil captures needs zstd: pip install zstandard (or Python 3.14)")
    return zstandard.ZstdDecompressor().stream_reader(stream)


# --- captures ---

@dataclass
class Segment:
    t: float  # Unix seconds
    protocol: str  # zone, chat, lobby
    dir: str  # out (server to client) or in
    src: int
    dst: int
    opcode: int
    raw: bytes  # the IPC segment, including its 16-byte IPC header

    @property
    def data(self) -> bytes:
        return self.raw[16:]


@dataclass
class Capture:
    path: Path
    id: str
    game_version: str
    writer: str
    start: datetime | None
    end: datetime | None
    censored: bool
    finished: bool

    @property
    def duration(self) -> float | None:
        if self.start and self.end and self.end > self.start:
            return (self.end - self.start).total_seconds()
        return None

    def _parts(self):
        if self.finished:
            archive = zipfile.ZipFile(self.path)
            return archive, _zstd_reader(archive.open("Data"))
        return None, open(self.path / "Data", "rb")

    def segments(self, protocols=("zone", "chat")):
        """Every IPC segment, in order."""
        archive, stream = self._parts()
        try:
            for message in _delimited(stream):
                try:
                    frame_message = _fields(message)
                    header = _fields(frame_message[1][0])
                    frame = frame_message[2][0]
                except (IndexError, ValueError):
                    break  # an unfinished capture's last frame
                protocol = PROTOCOLS.get(header.get(1, [0])[0], "?")
                if protocol not in protocols or len(frame) < 40:
                    continue
                direction = "out" if header.get(2, [0])[0] == 1 else "in"
                time_ms, = struct.unpack_from("<Q", frame, 16)
                count, = struct.unpack_from("<H", frame, 30)
                offset = 40
                for _ in range(count):
                    if offset + 16 > len(frame):
                        break
                    size, src, dst, kind = struct.unpack_from("<IIIH", frame, offset)
                    if size < 16:
                        break
                    if kind == 3 and size >= 32:
                        raw = frame[offset + 16:offset + size]
                        opcode, = struct.unpack_from("<H", raw, 2)
                        yield Segment(time_ms / 1000, protocol, direction, src, dst, opcode, raw)
                    offset += size
        finally:
            stream.close()
            if archive:
                archive.close()


def open_capture(path: str | Path) -> Capture:
    path = Path(path)
    finished = path.is_file()
    if finished:
        with zipfile.ZipFile(path) as archive:
            version_info, capture_info = archive.read("VersionInfo"), archive.read("CaptureInfo")
    else:
        version_info, capture_info = (path / "VersionInfo").read_bytes(), (path / "CaptureInfo").read_bytes()
    version = _fields(next(_delimited(io.BytesIO(version_info)), b""))
    info = _fields(next(_delimited(io.BytesIO(capture_info)), b""))
    versions = [v.decode() for v in version.get(6, [])]
    return Capture(
        path=path,
        id=info.get(1, [path.stem.encode()])[0].decode(),
        game_version=versions[0] if versions else "",
        writer=b" ".join(version.get(7, []) + version.get(8, [])).decode(),
        start=_timestamp(info.get(2, [None])[0]),
        end=_timestamp(info.get(3, [None])[0]),
        censored=bool(info.get(4, [0])[0]),
        finished=finished,
    )


def find_captures(paths) -> list[Capture]:
    """Captures in these files and folders (a Chronofoil folder, single captures, unfinished capture folders), one
    per capture ID (the censored copy when both are there), oldest first."""
    found: dict[str, Capture] = {}

    def add(path: Path):
        try:
            capture = open_capture(path)
        except (OSError, zipfile.BadZipFile, KeyError, IndexError, ValueError):
            return
        current = found.get(capture.id)
        if current is None or (capture.censored and not current.censored):
            found[capture.id] = capture

    for path in map(Path, paths):
        if path.is_file():
            add(path)
        elif (path / "Data").exists():
            add(path)
        elif path.is_dir():
            for child in sorted(path.iterdir()):
                if child.suffix in (".cfcap", ".ccfcap") or (child / "Data").exists():
                    add(child)
    epoch = datetime.min.replace(tzinfo=timezone.utc)
    return sorted(found.values(), key=lambda c: c.start or epoch)


# --- opcodes ---

def _current_opcodes() -> tuple[str, dict]:
    """Kawari's own table (opcodes.yml and the current fork opcodes), for the version it supports."""
    import yaml

    data = REPO / "resources" / "data"
    constants = yaml.safe_load((data / "constants.yml").read_text())
    version = constants["SUPPORTED_GAME_VERSION"]
    table: dict = defaultdict(dict)
    for kind, entries in yaml.safe_load((data / "opcodes.yml").read_text()).items():
        for entry in entries or []:
            if entry.get("opcode") is not None:
                table[kind][entry["opcode"]] = (entry["name"], entry.get("size"), "kawari")
    fork = yaml.safe_load((data / "fork_opcodes.yml").read_text()) or {}
    for kind, entries in fork.items():
        for entry in entries or []:
            if entry.get("opcode") is not None and entry.get("found_on") == version:
                table[kind].setdefault(entry["opcode"], (entry["name"], entry.get("size"), "fork"))
    return version, table


def load_opcodes() -> dict[str, dict[str, dict[int, tuple]]]:
    """{game version: {kind: {opcode: (name, size, how)}}}: the saved tables for older versions, plus Kawari's own
    for the version it supports."""
    tables: dict = {}
    if OPCODES_FILE.exists():
        saved = json.loads(OPCODES_FILE.read_text())
        for version, kinds in saved["versions"].items():
            tables[version] = {
                kind: {int(op): (ALIASES.get(e[0], e[0]), e[1], e[2]) for op, e in entries.items()}
                for kind, entries in kinds.items()
            }
    try:
        version, table = _current_opcodes()
        tables[version] = dict(table)
    except (ImportError, OSError, KeyError):
        pass
    return tables


def save_current_opcodes() -> str:
    """Freeze Kawari's current table into the saved file (run before merging a new patch's opcodes, so captures from
    this version stay readable.)"""
    version, table = _current_opcodes()
    saved = json.loads(OPCODES_FILE.read_text()) if OPCODES_FILE.exists() else {"versions": {}}
    saved["versions"][version] = {
        kind: {str(op): list(entry) for op, entry in sorted(entries.items())} for kind, entries in table.items()
    }
    OPCODES_FILE.write_text(json.dumps(saved, indent=1, sort_keys=True) + "\n")
    return version


class Names:
    """Packet names for one capture's game version."""

    def __init__(self, tables: dict, version: str):
        self.table = tables.get(version, {})
        self.known = bool(self.table)
        self.ops: dict[tuple[str, str], int] = {}
        for kind, entries in self.table.items():
            for op, (name, _, _) in entries.items():
                self.ops[(kind, name)] = op

    def name(self, segment: Segment) -> str:
        kind = KINDS.get((segment.protocol, segment.dir))
        entry = self.table.get(kind, {}).get(segment.opcode)
        return entry[0] if entry else f"Unknown{segment.opcode}"

    def size(self, segment: Segment) -> int | None:
        entry = self.table.get(KINDS.get((segment.protocol, segment.dir)), {}).get(segment.opcode)
        return entry[1] if entry else None


# --- what's in a capture ---

EFFECTS = ("ActionEffect1", "AoeEffect8", "AoeEffect16", "AoeEffect24", "AoeEffect32")


@dataclass
class Visit:
    """From one ZoneInit to the next: a zone or duty, and what happened there."""

    start: float
    end: float
    territory: int
    content: int  # ContentFinderCondition, 0 outside duties
    instanced: bool
    first: int = 0  # position of its ZoneInit among the capture's zone packets
    last: int = 0
    counts: Counter = field(default_factory=Counter)
    directors: set = field(default_factory=set)
    npcs: set = field(default_factory=set)  # (BNpcBase, BNpcName) of the NPCs spawned


@dataclass
class Summary:
    capture: Capture
    visits: list[Visit]
    known_opcodes: bool
    size_mismatches: Counter  # packet name -> how many didn't have the table's size


def summarize(capture: Capture, tables: dict) -> Summary:
    names = Names(tables, capture.game_version)
    visits: list[Visit] = []
    mismatches: Counter = Counter()
    current: Visit | None = None
    for position, segment in enumerate(capture.segments(protocols=("zone",))):
        name = names.name(segment)
        expected = names.size(segment)
        if expected is not None and len(segment.data) != expected:
            mismatches[name] += 1
        if segment.dir == "out" and name == "ZoneInit" and len(segment.data) >= 16:
            territory, instance, content = struct.unpack_from("<HHH", segment.data, 2)
            flags, = struct.unpack_from("<H", segment.data, 18)
            if current:
                current.end = segment.t
            current = Visit(segment.t, segment.t, territory, content, bool(flags & 0x2), position, position)
            visits.append(current)
            continue
        if current is None:
            continue
        current.end = segment.t
        current.last = position
        if segment.dir != "out":
            continue
        if name in ("ActorCast", "SpawnNpc", "SpawnPlayer", "DirectorVars", "DirectorPopupMessage", "NpcYell",
                    "EventStart", "DirectorMapEffect") or name in EFFECTS:
            current.counts["Effects" if name in EFFECTS else name] += 1
        if name == "DirectorVars" and len(segment.data) >= 4:
            current.directors.add(struct.unpack_from("<I", segment.data, 0)[0])
        if name == "SpawnNpc" and len(segment.data) >= 0x48:
            # SpawnNpc: 16 bytes, then CommonSpawn, whose base_id and name_id are 48 bytes in (spawn_npc.rs.)
            current.npcs.add(struct.unpack_from("<II", segment.data, 0x40))
    return Summary(capture, visits, names.known, mismatches)


def _clock(seconds: float | None) -> str:
    if seconds is None:
        return "?"
    seconds = int(seconds)
    return f"{seconds // 3600}:{seconds % 3600 // 60:02}:{seconds % 60:02}"


def load_names(path: str | Path | None) -> dict:
    """Optional names for territories and duties: {"territory": {id: name}, "content": {id: name}}, from the game's
    data (never committed)."""
    if not path:
        return {"territory": {}, "content": {}}
    raw = json.loads(Path(path).read_text())
    return {key: {int(k): v for k, v in raw.get(key, {}).items()} for key in ("territory", "content")}


def index_report(summaries: list[Summary], names: dict | None = None, duties_only: bool = False,
                 min_minutes: float = 0, numbers: list[int] | None = None) -> str:
    """The index: every capture (numbered as in the full list, for --export) and its visits."""
    names = names or {"territory": {}, "content": {}}
    lines = []
    for n, summary in zip(numbers or range(len(summaries)), summaries):
        capture = summary.capture
        start = capture.start.strftime("%Y-%m-%d %H:%M UTC") if capture.start else "?"
        state = ("censored" if capture.censored else "uncensored") + ("" if capture.finished else ", unfinished")
        lines.append(f"## {n}: {start}, {_clock(capture.duration)} ({capture.id[:8]}, game {capture.game_version}, "
                     f"{state})")
        if not summary.known_opcodes:
            lines.append(f"  No opcode table for game version {capture.game_version}: zones can't be read.")
        bad = {k: v for k, v in summary.size_mismatches.items() if v > 10}
        if bad:
            lines.append("  Packets not the size their table says (the table may be wrong for this version): "
                         + ", ".join(f"{k} ×{v}" for k, v in sorted(bad.items())))
        for v, visit in enumerate(summary.visits):
            minutes = (visit.end - visit.start) / 60
            if duties_only and not visit.content:
                continue
            if minutes < min_minutes:
                continue
            when = datetime.fromtimestamp(visit.start, timezone.utc).strftime("%H:%M")
            zone = names["territory"].get(visit.territory, "")
            duty = f" duty {visit.content}" + (f" {names['content'][visit.content]}" if visit.content in names["content"] else "") if visit.content else ""
            counts = visit.counts
            what = ", ".join(f"{k} {counts[k]}" for k in ("SpawnNpc", "ActorCast", "Effects", "DirectorVars",
                                                            "DirectorPopupMessage", "NpcYell") if counts.get(k))
            lines.append(f"  {n}.{v} {when} {minutes:5.1f} min  territory {visit.territory} {zone}{duty}"
                         + (" (instanced)" if visit.instanced and not visit.content else "")
                         + (f"  [{what}]" if what else ""))
        lines.append("")
    return "\n".join(lines)


def export(capture: Capture, tables: dict, visit: Visit, out: str | Path) -> int:
    """Write one visit's zone packets as a Kawari packet capture (JSON lines, see capture.py), without chat, so
    questkit's other tools read it. Returns the number of packets written."""
    names = Names(tables, capture.game_version)
    out = Path(out)
    opener = gzip.open if out.suffix == ".gz" else open
    written = 0
    with opener(out, "wt", encoding="utf-8") as f:
        for position, segment in enumerate(capture.segments(protocols=("zone",))):
            if position < visit.first:
                continue
            if position > visit.last:
                break
            name = names.name(segment)
            if name in PRIVATE:
                continue
            f.write(json.dumps({
                "t": int(segment.t * 1000), "dir": segment.dir, "src": segment.src, "dst": segment.dst,
                "name": name, "size": len(segment.data), "hex": segment.raw.hex(), "decoded": "",
                "source": f"chronofoil {capture.id} {capture.game_version}",
            }) + "\n")
            written += 1
    return written
