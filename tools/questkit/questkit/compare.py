"""questkit compare: what Kawari sent for a quest (a packet capture) against what retail did (a Recorder export).

For each quest in both, it lines up the quest's states (sequence and variables after each change, and completion)
and its notices, and says where Kawari differs: a state retail had that Kawari never reached, one Kawari had that
retail didn't, or a notice with other arguments. Several UpdateQuest packets sent together count as one change, as the
client only sees where they end up.
"""

from __future__ import annotations

import difflib
from dataclasses import dataclass, field

from . import capture
from .recording import Recordings

COLLAPSE_SECONDS = 0.3


@dataclass
class Run:
    """One go at a quest: its states in order, and its notices."""

    actor: int
    states: list[tuple] = field(default_factory=list)  # (sequence, vars hex) or ("completed",)
    notices: list[tuple] = field(default_factory=list)  # (notice, (args))
    state_times: list[float] = field(default_factory=list)
    notice_times: list[float] = field(default_factory=list)
    start: float = 0.0
    accepted: bool = False


def _state_key(sequence: int, variables: bytes) -> tuple:
    return (sequence, variables.hex())


def kawari_runs(packets: list[capture.Packet], quest: int) -> list[Run]:
    """The quest's runs in a capture, per player. A run starts when the quest is accepted (or at the capture's first
    sight of it) and ends when it's finished."""
    states = [s for s in capture.quest_states(packets) if s.quest == quest and s.kind != "list"]
    notices = [n for n in capture.notices(packets) if n.quest == quest]
    runs: list[Run] = []
    current: dict[int, Run] = {}
    pending: dict[int, tuple] = {}  # actor -> (time, key) of a change that may still be overwritten
    for state in states:
        run = current.get(state.actor)
        if state.kind == "accepted" or run is None:
            if run is not None:
                _flush(run, pending, state.actor)
            run = Run(state.actor, start=state.t, accepted=state.kind == "accepted")
            current[state.actor] = run
            runs.append(run)
            if state.kind == "accepted":
                continue
        if state.kind == "finished":
            _flush(run, pending, state.actor)
            run.states.append(("completed",))
            current.pop(state.actor, None)
            continue
        key = _state_key(state.sequence, state.variables)
        last = pending.get(state.actor)
        if last is not None and state.t - last[0] > COLLAPSE_SECONDS:
            _flush(run, pending, state.actor)
        pending[state.actor] = (state.t, key)
    for actor, run in current.items():
        _flush(run, pending, actor)
    for run in runs:
        end = None
        for other in runs:
            if other.actor == run.actor and other.start > run.start:
                end = other.start if end is None else min(end, other.start)
        run.notices = [(n.notice, tuple(n.args)) for n in notices
                       if n.actor == run.actor and n.t >= run.start - 1 and (end is None or n.t < end)]
    return runs


def _flush(run: Run, pending: dict, actor: int):
    entry = pending.pop(actor, None)
    if entry is None:
        return
    key = entry[1]
    if not run.states or run.states[-1] != key:
        run.states.append(key)


def retail_run(recordings: Recordings, quest: int) -> Run | None:
    events = recordings.quest_events(quest)
    if not events:
        return None
    run = Run(0, start=events[0].time, accepted=events[0].kind == "accepted")
    for event in events:
        key = ("completed",) if event.kind == "completed" else _state_key(event.sequence, event.variables)
        if not run.states or run.states[-1] != key:
            run.states.append(key)
            run.state_times.append(event.time)
    notices, covered = recordings.notices(quest)
    run.notices = [(n.notice, tuple(n.args)) for n in notices] if covered else None
    run.notice_times = [n.time for n in notices] if covered else []
    return run


def show(key) -> str:
    if key[0] == "completed":
        return "completed"
    sequence, hex_vars = key
    return f"seq {sequence:3} vars {' '.join(hex_vars[i:i + 2] for i in range(0, 12, 2))}"


def show_notice(notice) -> str:
    return f"notice {notice[0]} {list(notice[1])}"


def _flags_only(retail_key, kawari_key) -> bool:
    """Two states with the same sequence that differ only in which BitFlag8s (UI8F) are set, or which counters went
    up first, with as many flags set: the same progress, reached in another order."""
    if retail_key[0] == "completed" or kawari_key[0] == "completed" or retail_key[0] != kawari_key[0]:
        return False
    retail_flags = bin(int(retail_key[1][10:12], 16)).count("1")
    kawari_flags = bin(int(kawari_key[1][10:12], 16)).count("1")
    return retail_flags == kawari_flags


def _diff(retail: list, kawari: list, render, kawari_finished: bool = True) -> tuple[bool, list[str], bool]:
    """A line-by-line diff. Returns whether they're the same, the lines, and whether every difference is only an
    order difference (see _flags_only.)"""
    matcher = difflib.SequenceMatcher(a=retail, b=kawari, autojunk=False)
    lines = []
    same = True
    order_only = True
    opcodes = matcher.get_opcodes()
    for index, (op, a1, a2, b1, b2) in enumerate(opcodes):
        if op == "equal":
            for item in retail[a1:a2]:
                lines.append(f"    = {render(item)}")
            continue
        last = index == len(opcodes) - 1
        if last and op == "delete" and not kawari_finished:
            for item in retail[a1:a2]:
                lines.append(f"    . {render(item)}   (retail; the capture ends before this)")
            continue
        retail_part, kawari_part = retail[a1:a2], kawari[b1:b2]
        paired = min(len(retail_part), len(kawari_part)) if op == "replace" else 0
        pairs = paired > 0 and all(_flags_only(r, k) for r, k in zip(retail_part[:paired], kawari_part[:paired]))
        if pairs:
            same = False
            for r, k in zip(retail_part[:paired], kawari_part[:paired]):
                lines.append(f"    - {render(r)}   (retail, as many flags set: another order?)")
                lines.append(f"    + {render(k)}   (Kawari)")
            retail_part, kawari_part = retail_part[paired:], kawari_part[paired:]
        ends_here = last and not kawari_part and not kawari_finished
        for item in retail_part:
            if ends_here:
                lines.append(f"    . {render(item)}   (retail; the capture ends before this)")
            else:
                same = False
                order_only = False
                lines.append(f"    - {render(item)}   (retail, missing on Kawari)")
        for item in kawari_part:
            same = False
            order_only = False
            lines.append(f"    + {render(item)}   (Kawari only)")
    return same, lines, order_only and not same


def compare(packets: list[capture.Packet], recordings: Recordings, quests: list[int] | None = None,
            actor: int | None = None) -> tuple[bool, str]:
    kawari_quests = sorted({s.quest for s in capture.quest_states(packets) if s.kind != "list"})
    wanted = quests or [q for q in kawari_quests if recordings.quest_events(q)]
    lines = []
    all_same = True
    if not wanted:
        return False, "No quest is in both the capture and the recording.\n"
    for quest in wanted:
        retail = retail_run(recordings, quest)
        runs = [r for r in kawari_runs(packets, quest) if actor is None or r.actor == actor]
        name = recordings.quest_name(quest)
        if retail is None:
            lines.append(f"{quest}: not in the recording")
            all_same = False
            continue
        if not runs:
            lines.append(f"{quest} {name}: not in the capture")
            all_same = False
            continue
        # The last run that was accepted in the capture, else the last one.
        full = [r for r in runs if r.accepted]
        run = (full or runs)[-1]
        partial = "" if run.accepted else " (the capture starts partway through the quest)"
        retail_states = retail.states
        since = retail.start - 1
        if not run.accepted:
            # Compare from where the capture picks up.
            for index, key in enumerate(retail_states):
                if run.states and key == run.states[0]:
                    retail_states = retail_states[index:]
                    since = retail.state_times[index] - 1
                    break
        finished = bool(run.states) and run.states[-1] == ("completed",)
        same_states, state_lines, order_only = _diff(retail_states, run.states, show, finished)
        if same_states:
            verdict = "states match" if finished or retail_states[-1:] == run.states[-1:] else \
                "states match as far as the capture goes"
        elif order_only:
            verdict = ("states differ only where things were probably done in another order than on retail "
                       "(questkit replay checks the same order exactly)")
        else:
            verdict = "STATES DIFFER"
        notice_lines: list[str] = []
        if retail.notices is None:
            notice_verdict = "notices not recorded on retail"
            same_notices = True
        else:
            retail_notices = [n for n, t in zip(retail.notices, retail.notice_times) if t >= since]
            same_notices, notice_lines, _ = _diff(retail_notices, list(run.notices), show_notice, finished)
            notice_verdict = "notices match" if same_notices else "NOTICES DIFFER"
        all_same = all_same and (same_states or order_only) and same_notices
        lines.append(f"{quest} {name} (player {run.actor}){partial}: {verdict}, {notice_verdict}")
        if not same_states or any(line.startswith("    .") for line in state_lines):
            lines.extend(state_lines)
        if not same_notices:
            lines.extend(notice_lines)
    return all_same, "\n".join(lines) + "\n"
