# FrameEnder/Kawari — Fork Roadmap

Baseline: fork is identical to `redstrate/Kawari` master at `8f65e4b` (workspace v7.56.1).
Everything below was checked against that commit.

## Progress

- **White Mage kit** (`white-mage-kit`): all 36 Conjurer/White Mage actions and 6 role actions, plus engine support for healing, MP costs, area-of-effect, DoTs/HoTs, the Lily gauge, and Mind-based healer damage. Also fixes every action wiping the instance's queued tasks, and refreshed buffs expiring early. Needs in-game verification of the Lily gauge layout and tick/flying text.

- **Quest progression**: quest sequences and variables (saved, and sent to the journal), kill credit for quests (`onBNpcKill`), quest-spawned enemies, and a shared scene-flow helper in `Global.lua`. Close to Home now completes for every class.
- **Gridania story quests**: all 20 quests after Close to Home are scripted (To the Bannock → Renewing the Covenant), and checked against the client's own quest scripts pulled with `kawari-questdump`: scene numbers, key-item variables, objective counters and object flags. Chasing Shadows, Spirithold Broken, Leia's Legacy and To Guard a Guardian skip their solo battles with a chat message.
- **Packet capture**: `packet_capture: true` (world config) records every packet between the server and each client to `captures/*.jsonl`, with quest changes noted. Kawari Web's Diagnostics page turns it on and ships captures, logs, saved quest state and Kawari Probe (Dalamud) snapshots to the private `kawari-diagnostics` repo.
- **kawari-questdump**: pulls Quest-sheet data and the client's compiled quest scripts (with a readable listing) for a range of quests, so scripts can be written from what the client expects. Dumps are game data and stay out of the repo (`quest-dumps/` is ignored).

## Where the code stands

| Area | State |
|---|---|
| Codebase | ~54k lines of Rust; the world server is ~36.5k of that. 406 TODO/FIXME markers. |
| Build | `cargo build -p kawari-world` builds clean. `cargo test --workspace` passes except `inventory::equipped::tests::test_item_level`, which needs a real game install. |
| Packets | 201 server-zone and 103 client-zone opcodes known; 88 of the 103 client packets have handlers. Plumbing is not the bottleneck — depth is. |
| Quests | 3 openings, Close to Home (all classes), and 20 Gridania story quests up to lv 14. |
| Duties | 2: Sastasha (`s1d1_re.lua`) and A Merchant's Tale (`m6d1.lua`). |
| FATEs | 2, and `CleverGirls` is a test script spawning a placeholder BNPC. |
| Jobs | Dancer and White Mage have real kits. Every other job has 1–2 starter skills. |
| Status effects | 2 scripts (Sprint, Jog). |

## Gaps, in priority order

### Tier 1 — Lua content (fast, low risk, mostly script work)

1. **Job kits.** Action scripts are small and templated (`POTENCY` constant + `EffectsBuilder`), and Dancer shows the pattern for combos, traits and procs. Level 1–50 kits for every job can be scaffolded from the Action sheet and then tuned.
2. **MSQ past the first quests in each city.** Gridania is scripted up to Renewing the Covenant (lv 14). Solo quest battles, quest event ranges, and event-action cast bars ("searching…") aren't supported yet, so scripts work around them.
3. **Status effect scripts** for the buffs/debuffs the new job kits apply.
4. **Real FATEs** replacing the test script, plus a template for adding more.

### Tier 2 — Rust systems with scaffolding already in place

1. **Cooldowns** — `server/action.rs` never sends the cooldown ActorControlSelf (TODO at line 474), and cast times ignore haste. Needed before job kits feel right.
2. **Crafting** — `events/crafting.rs` is a stub: any action jumps progress straight to max. Needs a real progress / quality / durability / CP simulation.
3. **Gathering** — uses placeholder EXP (`events/gathering.rs`).
4. **Duty completion** — `server/director.rs`: completion isn't saved per player, vote-abandon and DirectorTrigger handling were removed in a refactor, and the director only waits for the first player to be ready.
5. **Housing persistence** — furniture positions and rotations aren't stored, dyes aren't implemented, ownership checks are missing.
6. **Pets** — Summoner/Scholar pet IDs and names are hardcoded in `server/action.rs`.

### Tier 3 — Needs retail packet captures (Chronofoil)

- Market board (handler returns a placeholder)
- Trade (stub to prevent softlocks)
- Blacklist, Fellowships (return empty lists)
- Retainers

## Fork maintenance

- Keep `upstream` as a remote and pull patch updates from it rather than redoing them.
- Keep fork additions mostly additive (new scripts, new modules) so upstream merges don't conflict with them.
- Fix or gate `test_item_level` so CI works without a game install.

## Working setup needed

Most content work reads game data (Action, Quest, ContentFinderCondition sheets, etc.), and in-game testing needs the client. Either link the Claude desktop app on the CachyOS server so the work can run against the real install, or export the specific sheets needed for each task.
