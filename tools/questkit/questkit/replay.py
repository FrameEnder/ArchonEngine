"""questkit replay: runs each quest script against its fixture (what retail did) and reports where they differ."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from . import fixtures, repo
from .harness import Harness, HarnessError


@dataclass
class StepResult:
    index: int
    step: dict
    ok: bool
    problems: list[str] = field(default_factory=list)
    got: dict = field(default_factory=dict)
    trace: list[str] = field(default_factory=list)


@dataclass
class QuestResult:
    quest: int
    script: Path | None
    steps: list[StepResult] = field(default_factory=list)
    error: str | None = None
    skipped: str | None = None

    @property
    def ok(self) -> bool:
        return self.error is None and self.skipped is None and all(s.ok for s in self.steps)


def describe(step: dict) -> str:
    for key in ("talk", "kill", "range", "enter_territory", "battle_won", "todo"):
        if key in step:
            value = step[key]
            what = f" ({step['what']})" if step.get("what") else ""
            return f"{key} {value}{what}"
    return "?"


def run(fixture: dict, script: Path | None = None) -> QuestResult:
    quest = fixture["quest"]
    script = script or repo.find_quest_script(quest)
    result = QuestResult(quest, script)
    if script is None or not Path(script).exists():
        result.error = f"no script for quest {quest} in {repo.QUEST_SCRIPTS}"
        return result
    try:
        harness = Harness(script, quest, fixture.get("params", {}))
    except HarnessError as error:
        result.error = str(error)
        return result
    harness.world.aetherytes.update(fixture.get("aetherytes", []))

    for index, step in enumerate(fixture.get("steps", [])):
        trace: list[str] = []
        problems: list[str] = []
        outcome: dict = {}
        if "todo" in step:
            result.steps.append(StepResult(index, step, False, [f"not filled in: {step['todo']}"]))
            break
        try:
            results = step.get("results", {})
            if "talk" in step:
                outcome = harness.talk(step["talk"], results, trace)
            elif "kill" in step:
                outcome = harness.kill(step["kill"][0], step["kill"][1], trace)
            elif "range" in step:
                outcome = harness.enter_range(step["range"], results, trace)
            elif "enter_territory" in step:
                outcome = harness.enter_territory(results, trace)
            elif "battle_won" in step:
                outcome = harness.battle_won(step["battle_won"])
            else:
                problems.append("a step needs one of talk, kill, range, enter_territory or battle_won")
        except HarnessError as error:
            problems.append(str(error))
        except Exception as error:  # Lua errors
            problems.append(f"script error: {str(error).strip().splitlines()[0] if str(error).strip() else error!r}")

        if not problems and outcome and not outcome.get("finished"):
            problems.append("the event was never finished (no finish_event): the client would be left waiting")

        expect = step.get("expect")
        state = harness.state()
        if expect and not problems:
            if expect.get("completed"):
                if not state.get("completed"):
                    problems.append(f"expected the quest to be completed, it's at sequence {state.get('sequence')} "
                                    f"vars {state.get('vars')}")
            else:
                if state.get("completed"):
                    problems.append("the quest was completed, retail wasn't done yet")
                elif "sequence" not in state:
                    problems.append("the quest isn't active")
                else:
                    if "sequence" in expect and state["sequence"] != expect["sequence"]:
                        problems.append(f"sequence {state['sequence']}, retail {expect['sequence']}")
                    if "vars" in expect and state["vars"] != expect["vars"]:
                        problems.append(f"vars {spaced(state['vars'])}, retail {spaced(expect['vars'])}")
            if "notices" in expect:
                got = outcome.get("notices", [])
                if got != expect["notices"]:
                    problems.append(f"notices {got}, retail {expect['notices']}")
        if "spawns" in step and not problems:
            if outcome.get("spawned", []) != step["spawns"]:
                problems.append(f"spawned {outcome.get('spawned', [])}, expected {step['spawns']}")
        result.steps.append(StepResult(index, step, not problems, problems, state, trace))
        if problems:
            break
    return result


def spaced(hex_text: str) -> str:
    return " ".join(hex_text[i:i + 2] for i in range(0, len(hex_text), 2))


def run_all(quests: list[int] | None = None) -> list[QuestResult]:
    results = []
    for path in fixtures.all_fixtures():
        fixture = fixtures.load(path)
        if quests and fixture["quest"] not in quests:
            continue
        results.append(run(fixture))
    return results


def format_results(results: list[QuestResult], verbose: bool = False) -> str:
    lines = []
    for result in results:
        name = result.script.stem if result.script else "?"
        if result.error:
            lines.append(f"FAIL {result.quest} {name}: {result.error}")
            continue
        total = len(result.steps)
        if result.ok:
            lines.append(f"ok   {result.quest} {name}: {total} steps match retail")
        else:
            failed = result.steps[-1]
            lines.append(f"FAIL {result.quest} {name}: step {failed.index + 1} ({describe(failed.step)})")
            for problem in failed.problems:
                lines.append(f"       {problem}")
            if failed.trace:
                lines.append(f"       the script did: {', '.join(failed.trace)}")
        if verbose:
            for step in result.steps:
                mark = "ok " if step.ok else "!! "
                state = step.got.get("vars") and f"seq {step.got.get('sequence')} vars {spaced(step.got['vars'])}" \
                    or ("completed" if step.got.get("completed") else "")
                lines.append(f"       {mark}{step.index + 1:2}. {describe(step.step):40} -> {state}")
    passed = sum(1 for r in results if r.ok)
    lines.append(f"{passed} of {len(results)} quests match their recordings")
    return "\n".join(lines)
