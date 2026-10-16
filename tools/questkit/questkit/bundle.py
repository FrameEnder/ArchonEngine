"""Reading a diagnostics bundle (kawari-diagnostics/bundles/<time>-<note>/, written by Kawari Web)."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

from . import capture
from .recording import parse_time

_LOG_LINE = re.compile(r"^(\d{4}-\d\d-\d\dT[\d:.]+Z)\s+(TRACE|DEBUG|INFO|WARN|ERROR)\s+(\S+?):\s?(.*)$")


@dataclass
class LogLine:
    time: float | None
    level: str
    target: str
    message: str
    raw: str


def parse_log(text: str) -> list[LogLine]:
    lines = []
    for raw in text.splitlines():
        match = _LOG_LINE.match(raw)
        if match:
            stamp, level, target, message = match.groups()
            try:
                when = parse_time(stamp)
            except ValueError:
                when = None
            lines.append(LogLine(when, level, target, message, raw))
        elif lines and raw.strip():
            # Continuation of the previous line (multi-line errors.)
            lines[-1].message += "\n" + raw
    return lines


class Bundle:
    def __init__(self, path: str | Path):
        self.path = Path(path)
        if not (self.path / "meta.json").exists():
            raise ValueError(f"{self.path} isn't a diagnostics bundle (no meta.json)")
        self.meta = json.loads((self.path / "meta.json").read_text())
        self.characters = self._json("characters.json") or []
        self.config = self._json("config.json") or {}

    @classmethod
    def resolve(cls, path: str | Path) -> "Bundle":
        """A bundle folder, or a kawari-diagnostics checkout (its latest.txt names the newest bundle.)"""
        path = Path(path)
        latest = path / "latest.txt"
        if not (path / "meta.json").exists() and latest.exists():
            name = latest.read_text().strip()
            candidate = Path(name) if Path(name).is_absolute() else path / name
            if not (candidate / "meta.json").exists():
                candidate = path / "bundles" / Path(name).name
            path = candidate
        return cls(path)

    def _json(self, name: str):
        file = self.path / name
        if file.exists():
            return json.loads(file.read_text())
        return None

    @property
    def note(self) -> str:
        return self.meta.get("note", "")

    @property
    def created(self) -> float | None:
        text = self.meta.get("created")
        if not text:
            return None
        return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()

    @property
    def quest_test(self) -> dict | None:
        """Set by Kawari Web's Quest Lab "Test this quest": which quest was being tested, on whom, from when."""
        return self.meta.get("quest_test")

    def snapshots(self) -> list[dict]:
        found = []
        for file in sorted((self.path / "snapshots").glob("*.json")):
            try:
                data = json.loads(file.read_text())
            except json.JSONDecodeError:
                continue
            data["_file"] = file.name
            found.append(data)
        return found

    def capture_files(self) -> list[Path]:
        return sorted((self.path / "captures").glob("*.jsonl*"))

    def packets(self) -> list[capture.Packet]:
        return capture.load_many(self.capture_files())

    def log(self) -> list[LogLine]:
        file = self.path / "server.log"
        if not file.exists():
            return []
        return parse_log(file.read_text(errors="replace"))
