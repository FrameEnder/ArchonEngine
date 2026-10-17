# FrameEnder/Kawari — Fork Roadmap

Forked from `redstrate/Kawari` master at `8f65e4b` (workspace v7.56.1); upstream merged through Codeberg `b1e7e2b` (patch 7.57).

Detailed documentation of everything below is in `docs/src/fork/` (start at `overview.md`); this file is the
short version and the plan.

## Progress

- **White Mage kit** (`white-mage-kit`): all 36 Conjurer/White Mage actions and 6 role actions, plus engine support for healing, MP costs, area-of-effect, DoTs/HoTs, the Lily gauge, and Mind-based healer damage.
- **Quests**: the quest engine (sequences, the six variable bytes laid out like the client's QuestWork, kill credit by BNpcBase, event actions, notices, quest-spawned enemies) and 16 Gridania quests from Close to Home (Conjurer) to Salvaging the Scene, most checked step by step against retail recordings. See `docs/src/fork/quests.md`.
- **Quest battles**: solo instances with QuestBattle directors; Chasing Shadows (11) and Spirithold Broken (15) timed from recordings.
- **Combat**: retail player stats and damage formula, recorded enemy HP and damage, enmity, aggro, leashing, area attacks, kill EXP and chains, drops.
- **Timing**: 20 ms task tick, slidecasting, enemy hits interrupt quest progress bars, Lua tasks from actions run immediately (Teleport).
- **Saving**: autosave every 30 s, dead connections dropped after 60 s, playtime saved as it goes, map discovery saved.
- **Travel**: attunement animation, Teleport unlocked by a second aetheryte.
- **Opcodes**: `fork_opcodes.yml`, read from `ffxiv_dx11.exe` at startup; `kawari-opcodescan`. Merged upstream through 7.57.
- **Tools**: `kawari-questdump` (dump, chain, extract, disasm, index, items), packet capture, `!rawpacket`, `!rawsweep`, `!directorscene`, `!forkopcodes`.
- **Kawari Web** (separate repo): Quest Lab, quest progress, Diagnostics, and the Shop (gil and play coins, delivered by moogle mail).

## Where the code stands

Size and packet counts are from the fork point; the content rows are current.

| Area | State |
|---|---|
| Codebase | ~54k lines of Rust; the world server is ~36.5k of that. 406 TODO/FIXME markers. |
| Build | `cargo build -p kawari-world` builds clean. `cargo test --workspace` passes except `inventory::equipped::tests::test_item_level`, which needs a real game install. |
| Packets | 201 server-zone and 103 client-zone opcodes known; 88 of the 103 client packets have handlers. Plumbing is not the bottleneck — depth is. |
| Quests | Upstream's 3 openings, plus the fork's Gridania story from Close to Home (Conjurer) to Salvaging the Scene (387), done one quest at a time against recordings. |
| Duties | 2: Sastasha (`s1d1_re.lua`) and A Merchant's Tale (`m6d1.lua`). |
| FATEs | 2, and `CleverGirls` is a test script spawning a placeholder BNPC. |
| Jobs | Dancer and White Mage have real kits. Every other job has 1–2 starter skills. |
| Status effects | 2 scripts (Sprint, Jog). |

## Gaps, in priority order

### Tier 1 — Lua content (fast, low risk, mostly script work)

1. **Job kits.** Action scripts are small and templated (`POTENCY` constant + `EffectsBuilder`), and Dancer shows the pattern for combos, traits and procs. Level 1–50 kits for every job can be scaffolded from the Action sheet and then tuned.
2. **MSQ past Salvaging the Scene**, one quest at a time from a new retail recording; Close to Home for Lancer and Archer; Limsa and Ul'dah.
3. **Status effect scripts** for the buffs/debuffs the new job kits apply.
4. **Real FATEs** replacing the test script, plus a template for adding more.

### Tier 2 — Rust systems with scaffolding already in place

0. **Palace of the Dead** on the `Deep-Dungeon` branch, after a Recorder 2.3 re-recording (notes in kawari-diagnostics).
1. **Cooldowns** — `server/action.rs` never sends the cooldown ActorControlSelf (TODO at line 474), and cast times ignore haste. Needed before job kits feel right.
2. **Crafting** — `events/crafting.rs` is a stub: any action jumps progress straight to max. Needs a real progress / quality / durability / CP simulation.
3. **Gathering** — uses placeholder EXP (`events/gathering.rs`).
4. **Duty completion** — `server/director.rs`: completion isn't saved per player, vote-abandon and DirectorTrigger handling were removed in a refactor, and the director only waits for the first player to be ready.
5. **Housing persistence** — furniture positions and rotations aren't stored, dyes aren't implemented, ownership checks are missing.
6. **Pets** — Summoner/Scholar pet IDs and names are hardcoded in `server/action.rs`.

### Tier 3 — Needs retail packet captures (Chronofoil)

- Market board: done from Bob's captures (every tradable item listed, buying; see docs/src/fork/marketboard.md). Selling isn't.
- Trade (stub to prevent softlocks)
- Blacklist, Fellowships (return empty lists)
- Retainers

## Fork maintenance

- Keep `upstream` as a remote and pull patch updates from it rather than redoing them.
- Keep fork additions mostly additive (new scripts, new modules) so upstream merges don't conflict with them.
- Fix or gate `test_item_level` so CI works without a game install.

## Working setup needed

Most content work reads game data (Action, Quest, ContentFinderCondition sheets, etc.), and in-game testing needs the client. Either link the Claude desktop app on the CachyOS server so the work can run against the real install, or export the specific sheets needed for each task.
