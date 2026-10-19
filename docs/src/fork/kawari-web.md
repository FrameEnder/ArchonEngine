# Kawari Web

Kawari Web (`FrameEnder/Kawari-Web`, Flask, currently **2.18.1**) runs beside the server in its own container. Its
README is the user guide (setup, pages, service, forges, builds); this page is about **how its features work** and
how they depend on the fork. Code: `kawari_core/` (logic, no Flask) and `kawari_web/` (blueprints, templates).

## What it touches on the server

| Data | Read/write | Used by |
|---|---|---|
| `login.db` | read/write | accounts (one account for the site and the game launcher), password changes |
| `world.db` | read; writes only for logged-out characters (and `mail`, and the `marketboard_*` settings) | profiles, Progress, Quest Lab character setup, Shop gil and delivery, Marketboard prices |
| `config.yaml` | read/write | Dashboard config editor, Diagnostics toggling `world.packet_capture` |
| The active server image | runs throwaway containers | `kawari-questdump` (Quest Lab, Progress index, Shop item list) |
| `<data>/quest-dumps/` | written by questdump | `quest-index.json`, `item-index.json`, `marketboard-categories.json`, per-quest dumps |
| `<data>/captures/` | read | Diagnostics bundles |
| The web app's own SQLite db | read/write | feed, notifications, settings, shop tables |

The rule from [Saving](saving.md#editing-a-character-from-outside) applies: the server saves a logged-in character
over the database, so direct edits only happen while the character is logged out. Mail is safe at any time.

## Pages (summary)

Home, How to Connect, Feed and profiles (character tile boards, private by default), Notifications, Settings,
**Shop**, and the admin **Dashboard**: server control, config editor, database health/migrations, backups, accounts,
Auracite import/export, branding, Quest Lab, Progress, Marketboard, Diagnostics, Updates (which fork/forge/commit the server is
built from), container logs.

## Shop (2.12.0)

`kawari_core/shop.py`, `kawari_web/shop.py` (blueprint at `/shop`), templates in `templates/shop/`.

**Listings.** Table `shop_listing` (category, name, description, contents, `price_gil`, `price_coins`, `sell_gil`,
`sell_coins`, `visible`, `retail_price`, patch, notes, position). Seeded once from `kawari_core/data/shop_seed.json`:
829 listings in 16 categories, the Online Store's catalogue gathered from public sources (Console Games Wiki, Square
Enix's Tales of Adventure guide, Eorzean Tavern), every price 1. After that the database is the source of truth; admins
can reset it to the seed.

**Contents and item linking.** `contents` is `[{"name", "qty", "item_id"?, "alt": [...]}]`. Contents are matched
against `item-index.json` (`kawari-questdump items`, run from Shop → Manage → *Read the item list from the game*):
case-insensitive by name, alternative names tried longest first (costume pieces carry their set prefix variants), and
`Name #12345` pins an ID when several items share a name. A listing whose items aren't all found isn't **ready** and
can't be bought; Manage shows a link report. 162 costume sets came without their pieces and need contents entered.

**Gil.** Taken from the character picked at the top of the shop: world.db `inventory.contents` →
`currency.gil.quantity`, in one `BEGIN IMMEDIATE` transaction together with the mail. Refused while the character is
online.

**Play coins** (name configurable, `shop_coin_name`). Per web account in `shop_wallet` (balance, `minutes_counted`,
`carry`, `earned_total`). Earned from the summed `time_played_minutes` of every character on the account at
`shop_coins_per_hour` (25): on each visit, new minutes × rate + carry is divided by 60, the remainder carried so no
playtime is lost to rounding. Playtime going down (a deleted character) is never taken back.
`shop_count_past_playtime` (on by default) credits playtime from before the wallet existed. Admin adjustments and
purchases are logged in `shop_coin_log`. Coins are deducted, then the mail is written, then the coin deduction is
committed, so a failed delivery costs nothing. Works while online (it's not character data).

**Delivery** as retail does: a **reward letter** (`mail.kind = 1`, `LetterType::Reward`) from sender `-1` (u64::MAX,
which the client shows as the World), with up to **5 items** attached (six slots, the last kept for gil), condition
30000, quantities split by stack size. The text is `shop_letter` with `{site}` and `{listing}`, stored as a JSON list
of UTF-8 bytes (≤ 600). Kawari finds a letter by (recipient, sender, timestamp), so each letter gets a unique second.
The server reads `mail` whenever a mailbox opens, so it works for logged-in characters.

**Admin**: edit/add/delete listings, bulk prices per category, rename/rate the coins, adjust balances, enable or
disable either currency, show/hide retail prices, change the letter, reseed. Buyers get a "shop" notification and a
purchase history.

What retail store items *do* (Tales of Adventure boosts, Fantasia, etc.) is not implemented on the server: they arrive
as items; whether using them works depends on the server's support for that item.

## Storage limits (2.16.0)

Every update and build switch backs up both databases (`~/.kawari/backups/*.tar.gz`) and snapshots every character
(`~/.kawari/characters/<content id>/*.json`); every edit, Probe sync and Auracite import snapshots that character
first. Nothing removed them, so each update left a little more on disk. `kawari_web/retention.py` caps them:

* **Backups**: the newest 20 by default (setting `keep_backups`), pruned whenever one is made. Backups that a build
  still on disk would restore when switched back to (its `leave_backup` in `versions.json`) are never pruned, nor
  the one just restored from.
* **Snapshots**: the newest 30 per character by default (`keep_snapshots`), pruned whenever that character gets one;
  the update's "Snapshot every character" step prunes every character.
* **Builds**: installs end with "Remove old builds", the existing Prune (unpinned builds beyond Builds to keep, and
  their image tags).
* 0 keeps everything. The limits, the space used and "Save and apply" are on Dashboard → Database (Storage limits);
  backups and a character's snapshots have checkboxes and **Delete ticked**.

Kawari Probe's diagnostics snapshots were already capped (200). Packet captures (`data/captures`) still are only
cleared by hand (Diagnostics).

## Phones (2.18.1)

Under 700 px wide the top bar is two short rows: the brand with the bell and account menus, then the site menu
scrolling sideways (it used to wrap into three rows); the bell and account menus open across the screen under the bar;
the dashboard's section menu is one scrolling row, scrolled to the current section (`static/app.js`). `static/style.css`
holds both, in `@media (max-width: 700px)` blocks at its end.

## Dependency mirrors (2.18.0)

Building from source fetches Kawari's git dependencies from GitHub (`Cargo.toml`):

| Repository | What it is for the server |
|---|---|
| `redstrate/physis` | Reads the game's files: the SqPack archives, Excel sheets, the zones' layouts (LGB/LVB: NPCs, event objects, map ranges), navmesh tiles, character customize data and save data. Kawari's `CustomizeData`, race and tribe types and layout reading come from it. |
| `redstrate/Icarus` | Generated, typed bindings for every Excel sheet (`QuestSheet`, `ActionSheet`, `TerritoryTypeSheet`…), one branch per game patch (`ver/2026.09.01.0000.0000` now), on top of physis. `gamedata.rs` and `kawari-questdump` read all game data through it; only the sheets listed in the workspace's `features` are compiled. |
| `redstrate/recastnavigation-rs-sys` | Rust bindings to Recast/Detour: building navmeshes from zone geometry (`generate_navmesh`) and path-finding for monsters and NPCs. |
| `recastnavigation/recastnavigation` | The C++ Recast/Detour library itself, a git submodule of the bindings. |

Updates → Dependency mirrors (`kawari_web/mirrors.py`) maps any of them to a mirror ("original mirror" per line). The
builder sets each as `git config --global url.<mirror>.insteadOf <original>` and exports
`CARGO_NET_GIT_FETCH_WITH_CLI=true`, so cargo's fetches and the submodule go through git and its rewrites;
`Cargo.toml` and `Cargo.lock` are untouched. A mirror has to have the locked commits (and for Icarus the patch's
branch) and be readable without signing in. Oodle isn't fetched by the build (it's uploaded in Setup).

## Quest Lab (Dashboard → Quests)

Runs `kawari-questdump` for chains, single quests, raw files and the index; shows each quest's variables, objectives,
listeners and rewards beside the fork script and the client listing; **script check** (variables the Quest sheet
doesn't have, scenes the client script doesn't contain, `-- NOTE: unverified` markers); copyable `//gm quest`
commands; **Set up a character** (logged out, snapshot first: completes prerequisites, taking one branch of "any one
of" — the quest's own city — raises the class to the quest level, leaves the quest ready, at a sequence, completed or
never started); a zip bundle of dumps, scripts and `Global.lua` with a manifest. Dumps stay private.

### Test this quest (2.13.0)

`kawari_web/questtest.py`, the *Test this quest* card on each quest's page:

* **Start the test**: runs the same character setup as *Set up a character* (`prepare_character`, logged out,
  snapshot first; ready to accept or in progress at a sequence), turns packet capture on through `set_capture` if it
  was off (config.yaml `world.packet_capture` and the capture folder; the server only reads it at startup, so it can
  restart the server), and saves the session in the web app's settings (`diag_quest_test`: quest, name, character,
  content_id, where it started, start time, whether capture was already on).
* While it runs, the quest page shows it with a note box, and Diagnostics shows a banner. Only one test at a time.
* **Finish the test and send it**: `build_bundle(..., quest_test=...)` with a window covering the test (+2 minutes),
  the chosen character, the note prefixed `Quest test <id> <name>`, and `quest_test` in `meta.json` and the README.
  Committed to kawari-diagnostics like any bundle. Optionally turns capture back off (only if the test turned it on).
* **Cancel without sending** drops the session.

`questkit triage` reads `quest_test` and filters its timeline to that quest; `questkit compare` checks the bundle's
capture against a retail recording.

## Quest progress (Dashboard → Progress)

Every quest from A Realm Reborn to Dawntrail from the quest index (format 2), Main Scenario when in ScenarioTree,
side quests by journal section. **Implemented** = the active build has
`resources/scripts/events/quest/<id/100>/<script>.lua`; **verified** = its header says it was checked against a retail
recording. Totals per expansion and a filterable list linking to Quest Lab.

**Misc Content** (2.17.0, `kawari_core/content_progress.py`, the second tab): every duty and open-world zone from
`content-index.json` (`kawari-questdump content`, built from the tab), per expansion. Dungeons, Trials, Raids,
Ultimate Raids, Deep Dungeons, V&C Dungeons and Guildhests (ContentType 2, 4, 5, 28, 21, 30, 3) get their own groups,
the rest share "Other duties". A duty is **scripted** when the active build has `resources/scripts/content/<short
code>.lua` (what `server/mod.rs` loads when the duty starts) and **has spawns** when a drop-in whose `appends` LGB is
in the duty territory's level folder (TerritoryType `Bg` without its last part) has battle NPCs, nonpop ones included
since directors spawn them; done = both, started = one. A zone has spawns the same way, counting each map once and
leaving out quest battle layers (`QB_*`). It measures what the build has, not how complete a script is.

## Marketboard (Dashboard → Marketboard)

2.15.0. Sets the prices of the server's marketboard ([Marketboard](marketboard.md)): `kawari_core/marketboard.py`
(prices, the tables) and `kawari_web/marketboard.py` (the pages).

* **Laid out like the board**: the four tabs from ItemSearchCategory.Category, each tab's categories (by the sheet's
  Order), each category's items, 100 to a page. The item list is `item-index.json`, which since this version also has
  each item's search category, rarity, equip and item level and whether it can be HQ, and
  `marketboard-categories.json` (ID, name, tab, order); both written by `kawari-questdump items`. An item list from
  before (no `search_category`) asks to be read again.
* **Edits** go straight to `world.db`: a tab's rates (every value, blank = built-in), a category's (blank = the tab's;
  all blank or *Use the tab's rates* deletes its row), items' own NQ/HQ prices (a form per page; blank removes them),
  and the listing settings. These tables are only read by the server, so writing them while it runs is safe.
* **Shows** each item's level (by the rates' level source), rarity, computed NQ/HQ prices (greyed in the boxes),
  its own prices, and how many have sold; a name search across every category; the latest sales.
* The prices are computed with the same formula as the server (`price`); checked against the server's test examples.
  Without the tables (a server from before the marketboard) the page says to update and start the server once.

## Diagnostics (Dashboard → Diagnostics)

Commits one bundle to the private `kawari-diagnostics` repository (fine-grained token, Contents read/write): recent
server log, packet captures (toggle here), chosen characters' saved quest state and position, Kawari Probe snapshots
(`POST /api/diag/snapshot` with the character-sync token), build and redacted config.
**Battles** (2.14.0): Kawari Recorder posts each retail battle to `POST /api/diag/battle?file=<name>&label=<label>`
(the same token; the body is the battle's `.json.gz`, read by the route itself up to 64 MB since the app limits
requests to 8 MB). It's kept in `<web>/diag/battles` (with `index.json`) and filed in the diagnostics repository's
`battles/` in a background thread; the page's Battles card lists them with *File now* for a failed or early one, and a
bundle's README names the battles received in its window. `/api/diag/ping` answers `accepts: diag-snapshot/1
diag-battle/1`. Can also point
`filesystem.navimesh_path` at `<data>/navimesh` so navmeshes survive rebuilds. Bundle format:
[Recorder and diagnostics](recorder-and-diagnostics.md).
