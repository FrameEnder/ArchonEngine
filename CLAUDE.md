# FrameEnder/Kawari

Bob's fork of Kawari (an FFXIV server emulator). Companion projects: Kawari Web (`FrameEnder/Kawari-Web`) and the
private `FrameEnder/kawari-diagnostics` (bundles, recordings' analyses, the Dalamud plugins).

## Read first

The fork's documentation is `docs/src/fork/` (an mdbook section, start at `overview.md`). It describes every fork
feature and mechanic, where it lives, how it was verified, and what's missing.

## Keep the documentation current

**Every change updates `docs/src/fork/` in the same commit:**

* the page for the feature it touches, or a new page added to `docs/src/SUMMARY.md` under "FrameEnder fork";
* `changelog.md`: an entry under today's date (write `—` for the commit's own hash; fill in the previous one's);
* `known-issues.md`: add what was found, remove what was fixed;
* `quests.md`'s table when a quest is added or re-verified.

New facts learned while debugging (packet layouts, client behaviour, retail numbers) go in the relevant page even
when no code changes.

## Rules

* Square Enix's files never go in git: `.luab` scripts, quest dumps (`quest-dumps/` is ignored), sheets,
  `ffxiv_dx11.exe`, recordings with game text.
* Don't hardcode game text; read names and text from game data at runtime. Recorded numbers are fine, with a comment
  saying where they came from.
* Fork opcodes go in `resources/data/fork_opcodes.yml`, never upstream's `opcodes.yml`.
* Minimal, focused changes; list every changed file when reporting.
* Work on `Latest-Quest`; merge to `master` only with Bob's OK. Palace of the Dead work goes on `Deep-Dungeon`.
* Don't run `cargo fmt` on the workspace (it reformats upstream files); `rustfmt --check --edition 2024` the files
  you changed.
* Quest scripts follow the pattern in `docs/src/fork/quests.md` and are checked against the client's script and a
  retail recording (simulate with lupa before Bob tests).
