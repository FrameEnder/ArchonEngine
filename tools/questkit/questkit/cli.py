"""Command line for questkit."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def _triage(args) -> int:
    from .bundle import Bundle
    from .triage import triage

    bundle = Bundle.resolve(args.bundle)
    text = triage(bundle, timeline_items=args.timeline)
    if args.write:
        out = bundle.path / "TRIAGE.md"
        out.write_text(text)
        print(f"Wrote {out}")
    else:
        sys.stdout.write(text)
    return 0


def _fixture(args) -> int:
    from . import fixtures
    from .recording import Recordings

    recordings = Recordings.open(args.recording)
    quests = recordings.quest_ids if args.quests == ["all"] else [int(q) for q in args.quests]
    status = 0
    for quest in quests:
        dump = fixtures.load_dump(Path(args.dumps), quest)
        if dump is None:
            print(f"{quest}: no dump in {args.dumps} (kawari-questdump dump {quest})")
            status = 1
            continue
        fixture = fixtures.build(quest, recordings, dump, source=args.source or "")
        path = fixtures.fixture_path(quest)
        if path.exists() and not args.force:
            print(f"{quest}: {path} exists, --force to replace it")
            continue
        fixtures.save(fixture, path)
        todo = sum(1 for step in fixture["steps"] if "todo" in step)
        print(f"{quest}: wrote {path} ({len(fixture['steps'])} steps{f', {todo} to fill in' if todo else ''})")
    return status


def _replay(args) -> int:
    from .replay import format_results, run_all

    quests = [int(q) for q in args.quests] if args.quests else None
    results = run_all(quests)
    print(format_results(results, verbose=args.verbose))
    return 0 if results and all(r.ok for r in results) else 1


def _compare(args) -> int:
    from . import capture
    from .bundle import Bundle
    from .compare import compare
    from .recording import Recordings

    files = []
    for source in args.capture:
        path = Path(source)
        if path.is_dir():
            bundle = Bundle.resolve(path)
            files.extend(bundle.capture_files())
        else:
            files.append(path)
    if not files:
        print("No packet captures found.")
        return 1
    packets = capture.load_many(files)
    recordings = Recordings.open(args.recording)
    quests = [int(q) for q in args.quests] if args.quests else None
    same, text = compare(packets, recordings, quests, args.actor)
    sys.stdout.write(text)
    return 0 if same else 1


def _scaffold(args) -> int:
    from . import fixtures, scaffold
    from .recording import Recordings
    from .replay import format_results, run

    dumps = Path(args.dumps)
    dump = fixtures.load_dump(dumps, args.quest)
    if dump is None:
        print(f"No dump of quest {args.quest} in {dumps}: kawari-questdump dump {args.quest}")
        return 1
    if args.fixture:
        fixture = fixtures.load(Path(args.fixture))
    elif args.recording:
        fixture = fixtures.build(args.quest, Recordings.open(args.recording), dump, source=args.source or "")
    elif fixtures.fixture_path(args.quest).exists():
        fixture = fixtures.load(fixtures.fixture_path(args.quest))
    else:
        print("Give a --recording (or a --fixture) to scaffold from.")
        return 1
    todo = [s for s in fixture["steps"] if "todo" in s]
    if todo:
        path = fixtures.save(fixture, Path(args.out_fixture) if args.out_fixture else None)
        print(f"The recording doesn't say what {len(todo)} step(s) were; fill them in in {path} and run again "
              f"with --fixture {path}.")
        return 1
    text, draft = scaffold.scaffold(args.quest, dumps, fixture)
    out = Path(args.out) if args.out else scaffold.default_path(dump)
    if out.exists() and not args.force:
        print(f"{out} exists; --force to replace it, or --out somewhere else.")
        return 1
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(f"Wrote {out}")
    fixture_out = Path(args.out_fixture) if args.out_fixture else fixtures.fixture_path(args.quest)
    if not fixture_out.exists() or args.force:
        fixtures.save(fixture, fixture_out)
        print(f"Wrote {fixture_out}")
    guessed = [s for s in draft.steps if s.scene_guessed]
    if guessed:
        print(f"{len(guessed)} scene(s) guessed (TODO in the script): steps "
              + ", ".join(str(s.index + 1) for s in guessed))
    result = run(fixture, out)
    print(format_results([result], verbose=args.verbose))
    return 0 if result.ok else 1


def _lua_syntax(args) -> int:
    """Compiles every Lua script (without running it) with Lua 5.4, the version the server embeds."""
    from . import repo
    from .harness import LuaRuntime

    if LuaRuntime is None:
        print("Needs lupa: pip install lupa")
        return 1
    lua = LuaRuntime()
    compile_source = lua.eval("function(source, name) local f, err = load(source, name, 't'); return err end")
    root = Path(args.path) if args.path else repo.SCRIPTS
    failures = 0
    count = 0
    for path in sorted(root.rglob("*.lua")):
        count += 1
        error = compile_source(path.read_text(errors="replace"), "@" + str(path.relative_to(repo.ROOT)))
        if error:
            failures += 1
            print(error)
    print(f"{count - failures} of {count} Lua scripts compile")
    return 1 if failures else 0


def _battle(args) -> int:
    from . import battle

    runs = battle.load_runs(args.recording)
    if args.where:
        needle = args.where.lower()
        runs = [r for r in runs if needle in f"{r.get('Zone', '')} {r.get('Content', '')} {r.get('TerritoryId', '')}".lower()]
    if not runs:
        print("No battles found (Recorder 2.4 or later records them, inside duties and quest battles)")
        return 1
    if args.list:
        for n, run in enumerate(runs):
            label = f" [{run['_label']}]" if run.get("_label") else ""
            print(f"{n}: {run.get('Start', '')} {run.get('Zone', '')} {run.get('Content', '')}{label} "
                  f"({len(run.get('Actors') or {})} actors, {len(run.get('Actions') or [])} actions) {run.get('_source', '')}")
        return 0
    if args.which == "all":
        chosen = runs
    else:
        chosen = [runs[int(n)] for n in args.which.split(",")]
    parts = []
    if len(chosen) > 1:
        summary = battle.across(chosen)
        if summary:
            parts.append(summary)
    if not args.summary_only or not parts:
        parts += [battle.report(run, npc_filter=args.npc) for run in chosen]
    text = "\n\n".join(parts)
    if args.out:
        Path(args.out).write_text(text + "\n")
        print(f"Wrote {args.out}")
    else:
        print(text)
    return 0


def _chronofoil(args) -> int:
    from . import chronofoil

    if args.save_opcodes:
        version = chronofoil.save_current_opcodes()
        print(f"Saved Kawari's opcodes for {version} to {chronofoil.OPCODES_FILE}")
        if not args.captures:
            return 0
    if not args.captures:
        print("Give a Chronofoil folder, captures (.cfcap/.ccfcap) or unfinished capture folders")
        return 1
    captures = chronofoil.find_captures(args.captures)
    if not captures:
        print("No Chronofoil captures found")
        return 1
    tables = chronofoil.load_opcodes()
    if args.dropin is not None:
        from . import chronofoil_dropin

        if not args.appends:
            print("--dropin needs --appends: the LGB the drop-in adds to (TerritoryType's Bg folder + planmap.lgb)")
            return 1
        numbers = range(len(captures)) if args.which == "all" else [int(n) for n in args.which.split(",")]
        seen = chronofoil_dropin.sightings([captures[n] for n in numbers], tables, args.dropin)
        dropin, skipped = chronofoil_dropin.build(seen, args.appends, exclude_bases=set(args.exclude_base),
                                                  one_spot=set(args.one_spot))
        out = args.out or f"dropin-{args.dropin}.json"
        chronofoil_dropin.write(dropin, out)
        objects = dropin["layers"][0]["objects"]
        print(f"Wrote {len(objects)} battle NPCs ({sum(o['data']['hostile'] for o in objects)} hostile) to {out}; "
              f"left out: {skipped or 'nothing'}")
        return 0
    if args.battle:
        import gzip
        import json

        from . import battle, chronofoil_battle

        number, _, visit = args.battle.partition(".")
        capture = captures[int(number)]
        chosen = chronofoil.summarize(capture, tables).visits[int(visit)]
        run = chronofoil_battle.run_from_visit(capture, tables, chosen)
        out = args.out or f"chronofoil-{capture.id[:8]}-{visit}.json.gz"
        if out.endswith(".md"):
            Path(out).write_text(battle.report(run) + "\n")
        else:
            Path(out).write_bytes(gzip.compress(json.dumps(run).encode()))
        print(f"Wrote {out} ({len(run['Actors'])} actors, {len(run['Actions'])} actions); "
              f"read it with: questkit battle {out}" if not out.endswith(".md") else f"Wrote {out}")
        return 0
    if args.export:
        number, _, visit = args.export.partition(".")
        capture = captures[int(number)]
        summary = chronofoil.summarize(capture, tables)
        chosen = summary.visits[int(visit)]
        out = args.out or f"chronofoil-{capture.id[:8]}-{visit}.jsonl.gz"
        written = chronofoil.export(capture, tables, chosen, out)
        print(f"Wrote {written} packets to {out}")
        return 0
    numbers = list(range(len(captures))) if args.which == "all" else [int(n) for n in args.which.split(",")]
    summaries = [chronofoil.summarize(captures[n], tables) for n in numbers]
    text = chronofoil.index_report(summaries, chronofoil.load_names(args.names), duties_only=args.duties,
                                   min_minutes=args.min_minutes, numbers=numbers)
    if args.out:
        Path(args.out).write_text(text)
        print(f"Wrote {args.out}")
    else:
        print(text)
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(prog="questkit", description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    p = commands.add_parser("triage", help="summarize a diagnostics bundle")
    p.add_argument("bundle", help="a bundle folder, or a kawari-diagnostics checkout (uses latest.txt)")
    p.add_argument("--timeline", type=int, default=60, help="how many quest/event packets to list per player")
    p.add_argument("--write", action="store_true", help="write TRIAGE.md into the bundle instead of printing")
    p.set_defaults(run=_triage)

    p = commands.add_parser("fixture", help="make replay fixtures from a Kawari Recorder export and quest dumps")
    p.add_argument("quests", nargs="+", help="quest numbers, or all")
    p.add_argument("--recording", action="append", required=True,
                   help="a Recorder export (zip or unpacked folder); repeat for several")
    p.add_argument("--dumps", required=True, help="kawari-questdump's output folder (quest-dumps)")
    p.add_argument("--source", help="where the recording came from, kept in the fixture")
    p.add_argument("--force", action="store_true", help="replace existing fixtures")
    p.set_defaults(run=_fixture)

    p = commands.add_parser("replay", help="run the quest scripts against their fixtures (what retail did)")
    p.add_argument("quests", nargs="*", help="only these quests")
    p.add_argument("-v", "--verbose", action="store_true", help="show every step")
    p.set_defaults(run=_replay)

    p = commands.add_parser("compare", help="compare a quest in a Kawari capture with the same quest on retail")
    p.add_argument("--capture", action="append", required=True,
                   help="a capture (.jsonl or .jsonl.gz), a bundle folder or a kawari-diagnostics checkout; repeatable")
    p.add_argument("--recording", action="append", required=True, help="a Recorder export (zip or folder); repeatable")
    p.add_argument("--actor", type=int, help="only this player (actor ID) in the capture")
    p.add_argument("quests", nargs="*", help="only these quests (default: every quest in both)")
    p.set_defaults(run=_compare)

    p = commands.add_parser("scaffold", help="draft a quest script from its dump and what retail did")
    p.add_argument("quest", type=int)
    p.add_argument("--dumps", required=True, help="kawari-questdump's output folder (quest-dumps)")
    p.add_argument("--recording", action="append", help="a Recorder export (zip or folder); repeatable")
    p.add_argument("--fixture", help="scaffold from this fixture instead of a recording")
    p.add_argument("--source", help="where the recording came from, kept in the fixture")
    p.add_argument("--out", help="where to write the script (default: its place in resources/scripts)")
    p.add_argument("--out-fixture", help="where to write the fixture (default: tools/questkit/fixtures)")
    p.add_argument("--force", action="store_true", help="replace an existing script/fixture")
    p.add_argument("-v", "--verbose", action="store_true", help="show every step of the check")
    p.set_defaults(run=_scaffold)

    p = commands.add_parser("battle", help="what recorded duties or quest battles show: waves, targeting, heals")
    p.add_argument("recording", nargs="+",
                   help="Recorder exports (zip), the Recorder's battles folder, battle files (.json.gz) or a battles.json")
    p.add_argument("--list", action="store_true", help="list the battles found")
    p.add_argument("--where", help="only battles whose zone, content or territory contains this")
    p.add_argument("--run", dest="which", default="-1",
                   help="which battles: indexes from --list (e.g. 3 or 2,5,7), default the last; 'all' for every one")
    p.add_argument("--summary-only", action="store_true",
                   help="with several runs of the same battle, only the totals across them")
    p.add_argument("--npc", help="only NPCs whose name or layout ID contains this")
    p.add_argument("--out", help="write the report to this file")
    p.set_defaults(run=_battle)

    p = commands.add_parser("chronofoil", help="what's in Chronofoil captures (retail sessions): zones, duties, fights")
    p.add_argument("captures", nargs="*",
                   help="a Chronofoil folder, captures (.cfcap, .ccfcap) or unfinished capture folders")
    p.add_argument("--duties", action="store_true", help="only duties (content finder conditions)")
    p.add_argument("--min-minutes", type=float, default=0, help="leave out visits shorter than this")
    p.add_argument("--capture", dest="which", default="all",
                   help="which captures, by their number in the index (e.g. 3 or 2,5); default all")
    p.add_argument("--names", help="a JSON file of territory and duty names from the game data (never committed)")
    p.add_argument("--export", metavar="N.V",
                   help="write visit V of capture N (numbers from the index) as a Kawari packet capture")
    p.add_argument("--battle", metavar="N.V",
                   help="turn visit V of capture N into a battle run for `questkit battle` (or a report, with --out x.md)")
    p.add_argument("--dropin", metavar="TERRITORY", type=int,
                   help="write the territory's battle NPCs (every spawn seen) as a drop-in; needs --appends")
    p.add_argument("--appends", help="the drop-in's LGB, e.g. bg/ffxiv/wil_w1/fld/w1f2/level/planmap.lgb")
    p.add_argument("--exclude-base", type=int, action="append", default=[],
                   help="leave out this BNpcBase (repeatable)")
    p.add_argument("--one-spot", type=int, action="append", default=[],
                   help="keep only one spawn point of this BNpcBase, e.g. a hunt mark (repeatable)")
    p.add_argument("--out", help="where to write the index, the export, the battle or the drop-in")
    p.add_argument("--save-opcodes", action="store_true",
                   help="save Kawari's current opcodes for its game version (before merging a new patch)")
    p.set_defaults(run=_chronofoil)

    p = commands.add_parser("lua-syntax", help="check that every Lua script compiles (Lua 5.4, like the server)")
    p.add_argument("path", nargs="?", help="a folder of scripts (default: resources/scripts)")
    p.set_defaults(run=_lua_syntax)

    args = parser.parse_args(argv)
    return args.run(args)
