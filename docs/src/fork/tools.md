# Tools and debug commands

The Python toolkit for quests and bundles (triage, replay, scaffold, compare) and the CI are on their own page:
[questkit and CI](questkit.md).

## kawari-questdump (`tools/questdump`)

Reads quest data and client scripts from the server's own game install. Output goes to `quest-dumps/` (`--out` to
change; gitignored, it's Square Enix's data). Finds the game like the server does (`config.yaml` in the current
folder), or `--game <game folder>`. Full guide: [Quest Dumps](../scripting/questdump.md).

| Command | Writes | Used for |
|---|---|---|
| `dump <id/script/name>…` | `quest.json` (Quest sheet row: variables with names, todos, listeners, rewards, `previous_join`, place, class category), the `.luab`, a listing (`--listing` for every instruction) | Writing and checking quest scripts |
| `chain <id> <n>` | The next `n` quests in the same journal genre | Dumping a stretch of the MSQ |
| `extract <path or file name>…` | Any game file. A bare name (e.g. `ManFst005Btl.luab`) is found in every game_script index by its name hash; missing paths are skipped so several guesses can be tried | Quest battle scripts, openings |
| `disasm <file.luab>` | A listing from the built-in Lua 5.1 disassembler | Scripts you already have |
| `index` | `quest-index.json` (format 2): every quest's name, level, genre, issuer, area, class restriction, prerequisites and how they combine, `main_scenario` (in ScenarioTree), `expansion_name`, journal `genre_name`/`category_name`/`section_name` | Kawari Web's quest search, prerequisite completion and Progress page |
| `items` | `item-index.json`: every named item's `id`, `name`, `description`, `category` (ItemUICategory), `icon`, `stack_size`, `unique`, `untradable`, `search_category`, `rarity`, `level_equip`, `level_item`, `can_be_hq`, in the server's language; and `marketboard-categories.json` (every ItemSearchCategory with a tab: `id`, `name`, `tab`, `order`, `icon`, `class_job`) | Kawari Web's Shop item linking and Marketboard page |
| `content` | `content-index.json` (format 1): `expansions` and `content_types` (ID → name); `duties`, every ContentFinderCondition with a name and content type (`id`, `name`, `short_code`, `content_type`, `territory`, `territory_bg`, `intended_use`, `level`, `level_sync`, `item_level`, `ex_version`, `in_duty_finder`, `high_end`, `pvp`, `sort_key`); `zones`, every open-world TerritoryType (intended use 1) with a place name (`id`, `name`, `region`, `internal_name`, `bg`, `ex_version`) | Kawari Web's Progress → Misc Content |

Kawari Web runs it in a throwaway container from the active server image (Quest Lab, Progress, Shop → Manage).

## kawari-opcodescan (`tools/opcodescan`)

`cargo run --release -p kawari-opcodescan -- <game folder or ffxiv_dx11.exe> [command]`. Runs the same finders as
the server's startup scan (see [Opcodes](opcodes.md)), and explores the executable:

| Command | Does |
|---|---|
| *(none)* | Runs every `fork_opcodes.yml` finder and prints what each reads |
| `switches [min cases]` | Lists switches with at least that many cases (default 200), e.g. packet dispatchers |
| `case <switch RVA> <opcode>` | Where a switch sends an opcode |
| `handler <function RVA>` | Follows calls back from a function to the switch cases that lead to it |
| `check-handlers <switch RVA> <ServerZoneIpcType…>` | Round-trips the handler method over every known opcode (7.57 server packets: 139 exact, 41 sharing a handler, 1 mismatch) |
| `fingerprint <out.json> <opcodes.yml> <fork_opcodes.yml> <game version>` | Records every zone opcode's fingerprints from this executable, the opcode files being right for it (the fork's entries found on that version only); see [Every opcode after a game update](opcodes.md#every-opcode-after-a-game-update) |
| `match <fingerprints.json>` | Finds the fingerprinted opcodes in this executable and prints `{list: {name: opcode, [candidates] or null}}` |
| `paths <dispatcher RVA>` | Each opcode's case and where its path ends, as the matcher follows it |

Keep the executable outside the repository.

## Packet capture

`packet_capture: true` in the world config (`world.packet_capture`, directory `packet_capture_dir`, default
`captures/`) writes every packet segment of each zone connection to `zone-<unix time>-<connection>.jsonl`, one JSON
object per line (`servers/world/src/capture.rs`):

| Field | Meaning |
|---|---|
| `t` | Unix time in ms |
| `dir` | `out` (server → client), `in` (client → server), or `note` (free text in `text`; supported by the capture, but nothing writes notes since the quest rollback) |
| `name`, `kind`, `size` | Kawari's name for the packet, segment kind, data size |
| `hex` | The IPC segment as sent, **including its 16-byte IPC header** |
| `decoded` | Kawari's reading of it (truncated if huge) |

Off by default; files grow fast. Kawari Web's Diagnostics page toggles it and ships captures in bundles.

## Debug commands (fork additions)

**`!` commands only work in zone chat** (Say, Shout, Yell, the default channel). Typed in Party, a Tell, a Linkshell,
the Free Company or the Novice Network, the message goes through the chat server, which doesn't look for commands, so
it's simply posted. When a command runs, the server log shows `Args passed to Lua command <name>`.

Typed in game chat; the full list is in [Debug Commands](../debug_commands.md).

| Command | Use |
|---|---|
| `!rawpacket <opcode> <hex>` | Send any IPC packet. How the QuestWork layout was mapped (marker bytes). |
| `!rawsweep <from> <to> <hex>` | Send the same data with every *unknown* opcode in a range, to find one the client reacts to. |
| `!directorscene <scene> [flags hex]` | Play a scene of the current director (found the 60001 victory fanfare). |
| `!retainer hire <name> [class/job] [level]` / `list` / `release <name>` | Hire a retainer, list them, let one go. See [Retainers](retainers.md#hiring). |
| `!forkopcodes` | List the fork's opcodes, which are known for this game version, and how to find the rest. |
| `!achievement <id>` / `clear <id>` / `check` | Complete an achievement (with the pop-up), take one away, or complete every one whose requirements are met (`check` says how many it completed). |

GM commands for quests (`//gm quest accept|cancel|incomplete|complete|sequence|inspect <id> …`) are in
[GM Commands](../gm_commands.md).

## Server log hints

* "Variables available in event …": the Quest sheet parameters a script got.
* Unknown client packets whose size matches an outstanding fork opcode name the likely packet and what to save.
* At startup: outstanding fork opcodes, and what the executable scan found.
* Removals of zone clients, with the reason.
