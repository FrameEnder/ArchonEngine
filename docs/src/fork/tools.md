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
| `extract <path or file name>…` | Any game file. A bare name (e.g. `OpeningGridania.luab`) is found in every game_script index by its name hash; missing paths are skipped so several guesses can be tried. The names have to be exact: quest battles' scripts aren't `<quest script>Btl.luab` (`ClsCnj002Btl.luab` and `ClsCnj003Btl.luab` weren't there, 2026-10-27) | Openings, scripts whose name is known |
| `search <text>…` | Every game_script file, in any folder, whose contents mention one of the texts (case aside), with its listing; each named from its own strings when one hashes to the index's name (the `<name> loaded` its main chunk prints, also with `_NNNNN` after it, as quest scripts are `ClsCnj003_00092.luab` but print `ClsCnj003 loaded`), its hash otherwise; `search-results.txt` says which folder (numbered ones like `game_script/quest/000` included) and index each came from. Reads all of game_script, so it takes a while | Scripts whose name isn't known: quest battles' (Kawari Web: Quest Lab → Scripts that mention) |
| `scripts custom\|opening [--server resources/scripts]` | Every client script of a kind a sheet names, under `<out>/<kind>/`: CustomTalk's (`custom/<id / 100>/<Name>.luab`: NPC conversations, menus and the shops they open) or Opening's; each with its listing (`.txt`) and its row (`.json`: row and handler ID, the Lua parameters), and `index.json` (each script's path, the server script of the same name, whether the server has it, whether the game had the file). `--server` dumps only those the server has a script for | `questkit clientcheck` (checking the server's scripts against the client's) |
| `disasm <file.luab>` | A listing from the built-in Lua 5.1 disassembler | Scripts you already have |
| `index` | `quest-index.json` (format 2): every quest's name, level, genre, issuer, area, class restriction, prerequisites and how they combine, `main_scenario` (in ScenarioTree), `expansion_name`, journal `genre_name`/`category_name`/`section_name` | Kawari Web's quest search, prerequisite completion and Progress page |
| `items` | `item-index.json`: every named item's `id`, `name`, `description`, `category` (ItemUICategory), `icon`, `stack_size`, `unique`, `untradable`, `search_category`, `rarity`, `level_equip`, `level_item`, `can_be_hq`, in the server's language; and `marketboard-categories.json` (every ItemSearchCategory with a tab: `id`, `name`, `tab`, `order`, `icon`, `class_job`) | Kawari Web's Shop item linking and Marketboard page |
| `classes` | `class-index.json` (format 1): `classes`, every ClassJob with a Role (`id`, `name`, `name_english`, `abbreviation`, `role`, `parent`, `primary_stat`), in the game's UIPriority order | Kawari Web's glamour API (the Glamour Collector's Class dropdown) |
| `content` | `content-index.json` (format 1): `expansions` and `content_types` (ID → name); `duties`, every ContentFinderCondition with a name and content type (`id`, `name`, `short_code`, `content_type`, `territory`, `territory_bg`, `intended_use`, `level`, `level_sync`, `item_level`, `ex_version`, `in_duty_finder`, `high_end`, `pvp`, `sort_key`); `zones`, every open-world TerritoryType (intended use 1) with a place name (`id`, `name`, `region`, `internal_name`, `bg`, `ex_version`) | Kawari Web's Progress → Misc Content |

Kawari Web runs it in a throwaway container from the active server image (Quest Lab, Progress, Shop → Manage).

## Glamour Collector (`tools/glamour-collector`)

A Chrome extension (Manifest V3, no build step) that collects Eorzea Collection glamours into a list and sends it to
Kawari Web, which writes the party bots' `glamours.txt` ([Party bots → Glamours](party-bots.md#glamours)), or exports
the file. Nothing of the site's is kept in git; the list is the user's, in the browser's `chrome.storage.local`.

* **Install:** `chrome://extensions` → Developer mode → Load unpacked → the `tools/glamour-collector` folder.
* **Kawari Web** (1.2.0; the popup's top section): the address as the browser reaches it, on the LAN or the tailnet
  (a bare `host:port` is taken as `http://`), and an administrator's sync token (Settings → Character sync).
  *Connect* asks Chrome for that address (`optional_host_permissions`; only the address given is granted), pings
  `GET /api/glamours/ping` and fetches the class list (`GET /api/glamours/classes`), kept as `classList` and fetched
  again quietly each time the popup opens. *Send to Kawari Web* (pressed twice: "Replace the server's list with N?")
  posts the whole list; *Load from Kawari Web* adds the server's glamours that aren't in the list (by title and
  gender). *Forget* drops the address, token, class list and Chrome's permission. The API is described on the
  [Kawari Web page](kawari-web.md#bot-glamours-dashboard--bot-glamours). `kawariweb.js` holds the calls.
* **On a glamour page** (`ffxiv.eorzeacollection.com/glamour/…`) a panel bottom right: *Add to Kawari list* reads
  the page and adds it (again on the same page reads it again, keeping the gender and classes chosen); *Copy page
  HTML* copies the gear section's HTML, for when a page reads wrong.
* **The toolbar popup** lists the entries with male / female / ANY counts. Each has a gender choice (ANY, Male,
  Female), its classes as chips (× removes one) with a dropdown to add one (the five role groups, then each role's
  classes and jobs from the server's class list, read from the game install, so the extension holds no class names;
  before connecting, only the role groups; none for any class; a chip the server's list doesn't have, such as
  "Gatherers", is marked), and its piece lines to edit (`Slot: <item> | <dye> | <second dye>`). *Export
  glamours.txt* downloads the list, *Copy list* copies it.

An entry as exported:

```text
== Vampire Reverie (ANY)
Class: <Black Mage>
Head: <item> | <dye>
Hands: <item> | <dye> | <undyed>
Weapon: <item>
```

The page reader (`parse.js`, `kawariParseGlamour`) follows the site's markup as read on 2026-10-23 from 21 glamour
pages through Claude in Chrome:

| What | Where on the page |
|---|---|
| Title | `h1.title` (the page's other `h1`s are the menu's, which is where the first version's "Browse all" came from) |
| Gender | The `.columns` row whose `span.subtitle` is "Fits": a `.tag` reading "Any Gender", "Female" or "Male" |
| Classes | The row "Intended for": an `img` with the job's abbreviation as `alt` and its name as `title` ("RDM" / "Red Mage"), a group ("Tanks", "Caster DPS" with the tag "Disciples of Magic", "Gatherers" / "Disciples of the Land"), or "ALL" / "All Classes" |
| Slots | A `.list.box` each, labelled `.gear-icon-box-slot-name`: HEAD, BODY, HANDS, LEGS, FEET, EARRINGS, NECKLACE, BRACELETS, RING, FACE, WEAPON, OFFHAND; a `.list-item` per piece |
| Item | `.list-item-title` (the icon's `alt` repeats the name) |
| Dyes | `.list-item-description .tag` chips with a `has-text-dye-*` dot, or "◯ Undyed"; the other chips are labels ("Optional", "Seasonal Item", "Online Store Exclusive"), which the first version took for dyes |

Of 20 pages from the newest listing, 13 fit any gender, 6 female and 1 male; 9 were for all classes. `format.js`
writes and reads the entries' text; `content.js` is the page's panel, `popup.js` the list.

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
| `layouts <opcodes.yml> <fork_opcodes.yml> <game version> <core/src> <out folder> [<ClientStructs folder>]` | Maps each zone packet's fields from the client's handlers and builders and compares them with Kawari's structs; see [Packet Layouts from the Client](packet-layouts.md) |

Keep the executable outside the repository.

## sdbrelay (`tools/sdbrelay`)

A TCP relay that records the game's settings backup (the save data bank, its own connection, which Chronofoil
doesn't record) on retail: see [Settings Backup](settings-backup.md#recording-it).

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
| `!spawn playernpc <class>` / `dismiss` | A party bot of that class (name or abbreviation) joins you; `dismiss` sends yours away. See [Party Bots](party-bots.md). |
| `!forkopcodes` | List the fork's opcodes, which are known for this game version, and how to find the rest. |
| `!achievement <id>` / `clear <id>` / `check` | Complete an achievement (with the pop-up), take one away, or complete every one whose requirements are met (`check` says how many it completed). |

GM commands for quests (`//gm quest accept|cancel|incomplete|complete|sequence|inspect <id> …`) are in
[GM Commands](../gm_commands.md).

## Server log hints

* "Variables available in event …": the Quest sheet parameters a script got.
* Unknown client packets whose size matches an outstanding fork opcode name the likely packet and what to save.
* At startup: outstanding fork opcodes, and what the executable scan found.
* Removals of zone clients, with the reason.
