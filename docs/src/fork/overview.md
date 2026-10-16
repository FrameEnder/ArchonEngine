# The FrameEnder fork

This section documents everything the FrameEnder fork of Kawari adds or changes, how each piece works, where it
lives in the code, what it can be used for, and what is still missing. It is the reference to read before changing
anything in the fork, and it is **updated with every change** (see [Keeping these pages current](#keeping-these-pages-current)).

The rest of this book is upstream Kawari's documentation and still applies; these pages only cover the fork.

## The projects

| Project | What it is | Where |
|---|---|---|
| **Kawari (fork)** | The server emulator itself (Rust): login, lobby, world, patch, web services. The fork adds quests, quest battles, combat and NPC behaviour from retail recordings, saving, opcode scanning and tools. | `github.com/FrameEnder/Kawari` |
| **Kawari Web** | A Flask site that runs beside the server: accounts, feed, character profiles, the Shop, and an admin Dashboard (server control, config, Quest Lab, quest progress, Diagnostics). | `FrameEnder/Kawari-Web` |
| **kawari-diagnostics** | A private repository that receives debugging bundles from Kawari Web, plus the Dalamud plugins (Kawari Probe, Kawari Recorder, Kawari Bestiary) and research notes. | `FrameEnder/kawari-diagnostics` |
| **Upstream Kawari** | Joshua Goins' (redstrate) project, now on Codeberg. The fork merges its patch updates. | `codeberg.org/redstrate/Kawari` |

How they fit together:

```text
 Retail client ── Kawari Recorder / Probe (Dalamud) ──▶ recordings, snapshots ─┐
                                                                               ▼
 Game client ◀──▶ Kawari world server ──▶ packet captures, server.log ──▶ Kawari Web ──▶ kawari-diagnostics bundle
                         ▲                                                    │
                         └──── world.db / login.db (shared) ◀─────────────────┘ (Shop mail, gil, quest setup)
```

## Branches

| Branch | Purpose |
|---|---|
| `master` | What Bob runs. Only updated with Bob's OK, normally by fast-forwarding from `Latest-Quest`. |
| `Latest-Quest` | Day-to-day work: quests, engine fixes, tools. Bob builds and tests from here first. |
| `Deep-Dungeon` | Palace of the Dead work, branched from `master` at `d28f9cf`. Nothing on it yet beyond master. |
| `upstream-sync`, `opcode-scan`, `white-mage-kit`, `fix/instance-cleanup` | Finished feature branches, already merged. |
| `diag/quest-layout`, `diag/zone-hang`, `quest/close-to-home`, `set_quest_sequence`, `Event-Test` | Old investigation branches from before the quest rollback (`3fb25fe`). Kept for reference only. |

Remotes in the working copy: `origin` (the fork), `upstream` (redstrate/Kawari on GitHub, frozen since the move to
Codeberg) and `codeberg` (`FrameEnder/kawari-upstream`, a GitHub mirror of the Codeberg repository, which is what
patch updates are merged from).

## How work is done

1. **Record retail first.** Behaviour is copied from what retail does, not guessed. The Kawari Recorder (Dalamud)
   records quests, duties, monsters, your own stats and actions, and the quest packets; Bob exports a zip.
   See [Recorder and diagnostics](recorder-and-diagnostics.md).
2. **Read what the client expects.** `kawari-questdump` pulls the Quest sheet and the client's own compiled quest
   script for a quest, so scene numbers and variables come from the game. See [Tools](tools.md).
3. **Implement**, as small and focused a change as possible, in the existing patterns.
4. **Verify offline** where possible: quest scripts are run in a Lua simulation (lupa) against the recording's
   variables and notices; Rust changes get `cargo build`/`cargo test` and `rustfmt --check` on the changed files only
   (running `cargo fmt` on the workspace reformats upstream files and must not be committed).
5. **Bob tests in game**, and sends a diagnostics bundle from Kawari Web when something is off. The bundle has the
   server log, packet captures, the saved quest state and Kawari Probe snapshots of what the client believed.
6. **Merge to master** only with Bob's OK.

## Rules that always apply

* **Square Enix's files never go in git.** That covers `.luab` client scripts, quest dumps, sheets, `ffxiv_dx11.exe`
  and recordings that contain game text. `quest-dumps/` is gitignored; the executable stays in a scratch folder.
* **Don't hardcode game text.** Names, dialogue, item names and descriptions are read from the game data at runtime
  (Quest text sheets, BNpcName, Item…). Numbers that the game data doesn't contain (recorded damage, HP, timings) are
  fine, with a comment saying where they came from.
* **Keep upstream merges easy.** Fork additions should be additive (new scripts, new modules, `fork_opcodes.yml`
  instead of editing `opcodes.yml`).
* **Changes are minimal and focused**, and every change lists the files it touched.

## Keeping these pages current

Every change to the fork updates these pages in the same commit:

* the page for the feature it touches (or a new page, added to `SUMMARY.md`);
* [Changelog](changelog.md): one entry with the date, commit and what changed;
* [Known issues](known-issues.md) when something is found or fixed;
* the quest table in [Quests](quests.md) when a quest is added or re-verified.

The fork's `CLAUDE.md` (repository root) repeats this rule for anyone working on the code.

## Pages

* [Quests](quests.md): the quest engine, Lua API, script pattern, verification, every scripted quest.
* [Quest battles and directors](quest-battles.md): solo duties, director scripting, Chasing Shadows, Spirithold Broken.
* [Combat and NPCs](combat.md): damage formulas, enemy AI, enmity, aggro, leashing, EXP, drops, the White Mage kit.
* [Timing, casting and tasks](timing.md): the server's ticks, cast resolution, slidecasting, event actions.
* [Saving and connections](saving.md): autosave, dead connections, playtime, zone changes.
* [Travel](travel.md): aetherytes, attunement, Teleport, map discovery, the Gridania opening.
* [Opcodes](opcodes.md): `fork_opcodes.yml`, reading opcodes from the executable, the event packets.
* [Tools and debug commands](tools.md): `kawari-questdump`, `kawari-opcodescan`, debug and GM commands, packet capture.
* [Kawari Web](kawari-web.md): what each part does, the Shop's mechanics, Quest Lab, progress, Diagnostics.
* [Recorder and diagnostics](recorder-and-diagnostics.md): the Dalamud plugins and the bundle format.
* [Research notes](research.md): content not implemented yet (Palace of the Dead, Blunderville).
* [Known issues](known-issues.md)
* [Changelog](changelog.md)
