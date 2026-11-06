# ArchonEngine ControlPanel

ArchonEngine ControlPanel (`FrameEnder/Kawari-Web`, Flask, currently **2.26.0** on the `UI-Redesign` branch, **2.24.2**
on `main`; it was Kawari Web until 2.26.0) runs beside the server in its own container. Its README is the user guide
(setup, pages, service, forges, builds); this page is about **how its features work** and how they depend on the fork.
Code: `kawari_core/` (logic, no Flask) and `kawari_web/` (blueprints, templates). The repository, the Python packages
(`kawari_web`, `kawari_core`), the container and the systemd unit keep their `kawari-web` names, so nothing installed
has to move.

## What it touches on the server

| Data | Read/write | Used by |
|---|---|---|
| `login.db` | read/write | accounts (one account for the site and the game launcher), password changes |
| `world.db` | read; writes only for logged-out characters (and `mail`, and the `marketboard_*` settings) | profiles, Progress, Quest Lab character setup, Shop gil and delivery, Marketboard prices |
| `config.yaml` | read/write | Dashboard config editor, Diagnostics toggling `world.packet_capture` |
| The active server image | runs throwaway containers | `kawari-questdump` (Quest Lab, Progress index, Shop item list) |
| `<data>/quest-dumps/` | written by questdump | `quest-index.json`, `item-index.json`, `marketboard-categories.json`, `class-index.json`, `action-index.json`, per-quest dumps |
| `<data>/data/glamours.txt` | read/write | Bot glamours (the party bots' list; the server reads it from `/kawari/data`) |
| `<data>/captures/` | read | Diagnostics bundles |
| The web app's own SQLite db | read/write | feed, notifications, settings, shop tables |

The rule from [Saving](saving.md#editing-a-character-from-outside) applies: the server saves a logged-in character
over the database, so direct edits only happen while the character is logged out. Mail is safe at any time.

## Pages (summary)

Home, How to Connect, Feed (called Community in the top bar from 2.19.0) and profiles (character tile boards, private by default), Notifications, Settings,
**Shop**, and the admin **Dashboard**: server control, config editor, database health/migrations, backups, accounts,
Auracite import/inspect/export (extended backups from 2.22.0), branding, Quest Lab, Progress, Marketboard, Bot glamours (2.21.0), Diagnostics, Battle replay (2.25.0), Updates (which fork/forge/commit the server is
built from), container logs.

## Auracite backups (2.22.0)

`kawari_core/auracite.py`; routes in `kawari_web/dashboard.py` (`/characters/import`, `/characters/inspect`,
`/characters/<id>/export`) and the Character sync API (`kawari_web/syncapi.py`).

* **Import** (Dashboard → Database) applies a backup to an existing character, section by section, after a snapshot.
  From 2.22.0 it refuses a character who is logged in on a running server (the world server would save over it), as
  the sync API already did.
* **Extended backups** from Auracite (Kawari): crystals, key items, the saddlebag and the glamour dresser, plates and
  armoire go with *Inventory*; quests in progress, completed levequests and legacy quests, map markers and the
  gathering log with *Quest progress*; the chocobo companion is a new section, *Chocobo companion* (allowed for
  Character sync, not on by default). Sections the client hadn't loaded are left alone. The whole `kawari_extended`
  object is then stored in world.db's `auracite_extended` table (`store_extended`; skipped on a database without it).
  See [Auracite Backups](auracite.md) for the member-by-member mapping.
* **The saddlebag on an older character** (2.22.3): `Dump.inventory` makes the two saddlebag pages (35 slots,
  kinds `SaddleBag1`/`SaddleBag2`) when the stored inventory has none, then fills them. Before, it filled only pages
  already there, and an inventory saved before the world server had a saddlebag has no `saddlebag` key, so the
  backup's saddlebag was dropped without a word (Lira Ormont, bundle 20261026-095541: her backup held 15 stacks).
  Characters imported before 2.22.3 get it by importing the backup again, logged out.
* **An empty saddlebag doesn't empty a full one** (2.22.4): a backup without `saddlebag_seen_at` (Auracite (Kawari)
  before 1.8.1.3002) whose saddlebag pages hold nothing leaves the server's saddlebag as it is when that holds
  items, with the note "saddlebag kept". Those plugins read the saddlebag whenever the client said it was loaded,
  and the client says so with nothing in it until the saddlebag has been opened that session: Lira Ormont's backup
  d9b52e55 (2026-10-24 21:05 UTC) has `saddlebag_loaded: true` and two empty pages, 40 minutes after 06ffbf5d held
  15 stacks, and importing it emptied her saddlebag (bundle 20261026-144730). A backup with `saddlebag_seen_at` is
  taken as it stands, empty or not.
* **Blank inventories** (2.22.3, `chardata._blank_inventory`, used for new characters from another character's
  shape): the saddlebag's pages are emptied and the glamour dresser, plates and armoire cleared (both were copied
  from the other character), and the bags' pages keep their `kind`. The bundled `inventory_template.json` now has
  the bags' kinds and the saddlebag; without the kinds the world server couldn't read it as it stood (it now fills
  in what a saved inventory lacks, [Saving](saving.md#reading-a-saved-inventory)).
* **The adventurer plate** (2.22.1): an extended backup's `adventurer_plate` goes with *Grand Company, title and
  mentor status* into the volatile row's plate, its save time and the portrait, and the export writes it back.
* **Inspect only** shows a backup's summary and, for an extended one, each section: in the backup or not, loaded or
  not, imported or only kept (`Dump.extended_report`), without writing anything.
* **Export** adds `kawari_extended`: the stored object (`stored_extended`) with Kawari's own data written over the
  members it keeps, so exporting and importing again changes nothing and the rest survives.

Character snapshots don't include the `auracite_extended` row (it isn't one of `chardata.CHARACTER_TABLES`, whose rows
character creation fills from column defaults); a restore leaves it as it is.

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

## Look and admin layout (2.19.0)

The redesign (branch `UI-Redesign` of Kawari-Web, not yet merged) starts with the parts every page shares:

* `static/style.css` is rewritten around design tokens: a dark slate theme, the light theme kept, status shown as
  coloured dots and callouts. Every class the templates already used is restyled, so pages that haven't been redone
  yet still work and pick up the new look.
* Fonts: Sora (headings), IBM Plex Sans (text), IBM Plex Mono (IDs, ports, logs), bundled as woff2 in
  `static/fonts` with their SIL Open Font License files. Nothing is fetched from outside, so the container still
  works offline.
* Icons are inline SVG (`templates/_icons.html`), replacing the emoji bell.
* `--on-accent`: `app.py`'s `on_accent()` picks dark or white text for anything painted in the Branding accent,
  whichever has more contrast. The default accent was `#5aa9f0` then (`#b170c2` since 2.26.0, see Branding below).
* Admin pages extend `templates/dashboard/_layout.html`: a sidebar (`dashboard/_nav.html`) grouped Server, Build &
  config, Game data, Content and Community, and a header with the page's group and title (pages can override the
  `page_title` and `page_actions` blocks). The page for an endpoint is looked up in the layout's table. Shop
  management and the listing editor use it too. *Announcements* links to the Overview's announce card for now.

The pages themselves are redone in later commits on the same branch, from the design canvas Bob approved.

2.19.1 fixed a 500 on Quest Progress: the layout kept its page lookup in a variable named `page`, which hid the
page number Quest Progress passes. The layout's variables are `_adm_*` now.

## Branding: the ArchonEngine ControlPanel name, logo and favicon (2.26.0)

Kawari Web became ArchonEngine ControlPanel. What a visitor sees by default:

* **Name**: `webdb.BRANDING_DEFAULTS["site_name"]` is "ArchonEngine ControlPanel" (the page titles, the top bar, the
  admin sidebar, the Home page's heading); the footer says "Powered by ArchonEngine" and How to connect "an
  ArchonEngine server". The panel's own messages (its start-up banner, the diagnostics repository's first README,
  Quest Lab's "Made by", the Auracite exporter it writes into a backup's metadata) use the name too.
* **Logo**: ArchonEngine's mark, Bob's image (2026-11-06) traced into one SVG path: `static/logo/mark.svg` (viewBox
  327 × 425, `#361B3D`, two outlines, 169 points). Traced with scikit-image (`find_contours` at half alpha,
  `approximate_polygon` at 0.35 px); rendered in Chromium it covers the image's pixels with an IoU of 0.977.
  `_icons.html`'s `brand_mark()` puts the same path inline wherever no logo is uploaded (top bar, admin sidebar,
  sign-in and register), filled with `--logo`: `#361b3d` on the light theme, `#cfacd7` (the same hue, light, 9.5:1
  on the dark ground) on the dark one, where the mark's own colour wouldn't show. It's sized by height
  (`aspect-ratio: 327 / 425`), as it's taller than wide.
* **Favicon**: `/favicon.ico` is the branding's emoji if one is set, else the uploaded logo, else
  `static/logo/favicon.svg` (the mark on a square viewBox, its `prefers-color-scheme: dark` rule switching to the
  tint), sent with `no-cache` so a change shows at once. `base.html` lists `static/logo/favicon-32.png` first, with
  its size, so browsers that can use the SVG take it, and an `apple-touch-icon` (180 px; the uploaded logo if there
  is one). The PNGs (the mark on a light `#f4f5f8` tile, so they show on dark tab bars too) were rendered from
  `mark.svg` in Chromium. The favicon is reachable before the first account exists (first-run setup lets it through).
  The default emoji is now empty; the Branding page's field says so.
* **Accent**: `#b170c2`, the mark's hue (288°) light enough for buttons on both themes (5.4:1 on the dark ground).

A panel already set up has its branding stored (the Branding page saves every field), so `webdb.init()` runs
`_migrate_branding` once: a stored name "Kawari", globe emoji or `#5aa9f0` accent, Kawari Web's defaults, becomes
today's default, and `branding_defaults = 2` in the `setting` table records that it ran; anything else saved is kept,
and setting one of the old values again afterwards sticks. Checked with Flask's test client (an emoji, an uploaded
logo, a logo whose file is gone, a restart after "Kawari" was set again) and screenshots of Home on both themes, the
admin Overview, Branding and sign-in, the 390 px phone header and the favicon in light and dark browsers.

The plugins and the Glamour Collector were renamed with it where they show the name: Kawari Probe 0.7.1 and Kawari
Recorder 2.7.1 (messages, the *Send to ArchonEngine ControlPanel* section; their config keys `KawariWebUrl` and
`KawariWebToken` are unchanged, so settings carry over) and the Glamour Collector 1.2.1 (its popup's section, *Send to
ControlPanel* and *Load from ControlPanel*, short enough for the popup's buttons). The world server's `!mark` reply
names it too.

## Player pages, announcements and shop pictures (2.20.0)

* **Player pages** rebuilt to the design: Home, How to connect, sign in/register, first run, errors, Community (the
  feed), posts, Notifications (`?kind=` filter), Settings, Profile. The styles are all in `static/style.css`, one
  section per area.
* **Character pop-up.** Profile shows each character as a slim card; **Full stats** opens every stat
  (`stats.character_detail`, which now also returns jobs grouped by role, every currency and the top classes). It's
  a `:target` overlay, so it works without JavaScript; with JavaScript it's driven by a class instead, because
  Chrome keeps `:target` matched after `history.replaceState`.
* **In game** (Home's players, Profile, Community's *Who's online*, the server's player count) is believed only
  while the container is known to run: `stats.server_up()`. `volatile.is_online` stays set after a crash, and
  without Podman there's no way to tell, so then nobody shows as in game.
* **Announcements** (Dashboard → Announcements) are recorded in the web app's `announcement` table (title, body,
  sender, delivered, accounts), since the notifications themselves can be cleared per account.
* **Shop pictures.** A listing's thumbnail is its cover upload, else the game icon of its first linked item
  (`item-index.json`'s `icon`). **Fetch item icons** runs `kawari-questdump extract ui/icon/<folder>/<id>_hr1.tex
  ui/icon/<folder>/<id>.tex …` (missing paths are skipped, so both are asked for), then
  `kawari_core/icons.py` turns each `.tex` into `<data>/web/item-icons/<id>.png`, preferring the high-resolution
  copy. It reads the 80-byte header (format at 4, width/height at 8 and 10, the first surface's offset at 28) and
  decodes BC1/BC2/BC3/BC5/BC7 with Pillow's BCn decoder and the uncompressed 8888, 8888X, 4444, 5551, L8 and A8
  formats itself. Uploaded preview images (up to 8 a listing) are in `shop_listing_image`; deleting a listing or
  resetting the shop deletes their files.

## Client compatibility (2.20.1)

Overview compares the client's `.ver` files (`ffxivgame.ver`, `boot/ffxivboot.ver`, `sqpack/exN/exN.ver`) with the
active build's supported versions (`SUPPORTED_GAME_VERSION` and friends, the *Supports* Updates shows), the same way the
patch server does (`versions.compare_client`). ArchonEngine ControlPanel's container mounts only the data folder, so the
game folder usually isn't visible to it: `kawari_web/clientver.py` then starts a short-lived container from the server's
image with the install's folder (the game path's parent, so `boot/` is included) mounted read-only at `/client`, and
`/bin/sh` prints each `.ver` file. The answer is cached for ten minutes (one minute after a failure); **Check again**
clears it. Without the Podman API socket or a server image it says so instead.

## Phones (2.18.1, changed in 2.19.0)

The profile's character pop-up fills the screen on a phone; every rebuilt page works at 390 px without sideways
scrolling.


From 2.19.0 the top bar's two-row phone layout starts under 760 px, and the dashboard's sidebar folds behind a
**Menu** button under 900 px (a checkbox, so no JavaScript); the scrolling section row below is gone with the tabs.


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


**Script Check** (2.23.0, `kawari_web/questlab.py` `script_check`, `kawari_core/clientcheck.py`): Quest Lab's check for
the scripts that aren't quests. *Dump and check* runs `kawari-questdump scripts custom` and `scripts opening` (the
second may fail without stopping the job) with `--server /kawari/resources/scripts` in Quest Lab's throwaway container
(the image keeps the build, resources included, in `/kawari`), into `<data>/quest-dumps/client-scripts/`, emptied first.
The page checks the active build's `resources/scripts` against it with `kawari_core/clientcheck.py`, a copy of the
fork's `tools/questkit/questkit/clientcheck.py` taking the scripts folder as an argument (keep the two the same), and
shows the scripts that need a look, those worth reading, those without a client script and those that agree; the
report (`/dashboard/quests/script-check/report.md`) and a bundle (`.../bundle`: the report, `client-scripts/`, the
server's scripts under `fork-scripts/`) download from it. It needs a build whose `kawari-questdump` has `scripts`
(fork commit `cf9c620`; the page reads the binary's usage text to tell). Named Script Check because Overview's
`client_check` endpoint already re-reads the client's version.

**Scripts that mention** (2.24.0, the Run kawari-questdump card): `kawari-questdump search <text>…` in the same
throwaway container, into `<data>/quest-dumps/extracted/` beside the extracted files, for client scripts whose names
aren't known (quest battles': Bob's 2026-10-27 extract of `ClsCnj002Btl.luab` and `ClsCnj003Btl.luab`, names guessed
from a pattern, found nothing). The button is off until the active build's binary has `search` in its usage text.
`/dashboard/quests/extracted.zip` downloads every extracted file in one zip.
## Quest progress (Dashboard → Progress)

Every quest from A Realm Reborn to Dawntrail from the quest index (format 2), Main Scenario when in ScenarioTree,
side quests by journal section. **Implemented** = the active build has
`resources/scripts/events/quest/<id/100>/<script>.lua`; **verified** = its header says it was checked against retail
(`kawari_core/progress.py`'s `header_status`): a "checked/verified … against …" sentence, however many comment lines it
runs over, naming a capture, a recording or a Kawari Recorder export, or the older "recording"/"recorded on retail";
"unverified" anywhere in the header makes it Implemented. Before 2.24.2 only the word "recording" counted, and the
fork's headers say "Chronofoil capture": 21 of the 178 quest scripts showed as verified, 176 do now (39, never checked,
and 3018, whose capture has only its offer, don't). The build has only `resources/`, so the questkit fixtures can't be
the test. Totals per expansion and a filterable list linking to Quest Lab.

**Misc Content** (2.17.0, `kawari_core/content_progress.py`, the second tab): every duty and open-world zone from
`content-index.json` (`kawari-questdump content`, built from the tab), per expansion. Dungeons, Trials, Raids,
Ultimate Raids, Deep Dungeons, V&C Dungeons and Guildhests (ContentType 2, 4, 5, 28, 21, 30, 3) get their own groups,
the rest share "Other duties". A duty is **scripted** when the active build has `resources/scripts/content/<short
code>.lua` (what `server/mod.rs` loads when the duty starts) and **has spawns** when a drop-in whose `appends` LGB is
in the duty territory's level folder (TerritoryType `Bg` without its last part) has battle NPCs, nonpop ones included
since directors spawn them; done = both, started = one. A zone has spawns the same way, counting each map once and
leaving out quest battle layers (`QB_*`); PvP maps listed as open world (a `Bg` under `/pvp/`: Wolves' Den Pier) have no monsters and aren't listed (2.22.2). It measures what the build has, not how complete a script is.

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

## Bot glamours (Dashboard → Bot glamours)

2.21.0, on `UI-Redesign`. The party bots' glamour list ([Party bots → Glamours](party-bots.md#glamours)) and the API
the Glamour Collector extension uses ([Tools](tools.md#glamour-collector-toolsglamour-collector)):
`kawari_web/glamours.py`.

* **The list** is `glamours.txt` in `settings.db_dir`, the host folder mounted at `/kawari/data` (the world server
  reads it from `$KAWARI_DATA`). It's written beside the old one and renamed over it, so the server never reads half
  a file; the one it replaces goes to `<web>/glamour-backups/glamours-<date>-<time>.txt` (the newest 20), which the
  page can restore. The server notices the new modification time, so no restart.
* **The class list** is `class-index.json` in the quest dumps folder, written by `kawari-questdump classes` (the
  page's *Read the class list from the game*, as a job like the other questdump steps): every ClassJob row with a Role
  (`id`, `name`, `name_english`, `abbreviation`, `role`, `parent`, `primary_stat`), in UIPriority order. The API groups
  it by role as Eorzea Collection does: Tanks (Role 1), Healers (4), Melee DPS (2), Physical Ranged DPS (3 with
  PrimaryStat 2, dexterity) and Caster DPS (3 otherwise), each class by its English name.
* **The API** takes the Companion sync token (Settings → Character sync), and only an administrator's (403
  otherwise), since the list changes what every bot wears:

| Call | Answer |
|---|---|
| `GET /api/glamours/ping` | `accepts: "glamours/1"`, whether the class list is there, the list's counts (`glamours`, `male`, `female`, `any`, `classes`) and modification time |
| `GET /api/glamours/classes` | `roles`, `groups` (`[{name, classes: [names]}]`) and `classes` (`[{id, name, abbreviation, group, parent}]`); 404 before the class list is read |
| `GET /api/glamours` | `text` (the file) and its counts |
| `POST /api/glamours` | `{"text": "..."}` (or the text as the body, up to 2 MB) replaces the list; refused (400) without a single `==` header |

The extension reaches ArchonEngine ControlPanel wherever the browser can, on the LAN or over Tailscale: it asks Chrome
for that address (an optional host permission), which also spares the API any CORS headers. Checked with Flask's test
client (no token 401, a non-admin's 403, a list written with its backup) and with the extension in Chromium against a
test server (connect, the dropdown from the class list, send, load).

## Battle replay (Dashboard → Battle replay, 2.25.0)

A retail fight and a Kawari one side by side on a top-down map, on one clock (`kawari_web/replay.py`,
`kawari_core/battle_replay.py`, `templates/dashboard/replay.html`). Made so a fight that goes wrong on Kawari can be
*seen* next to retail's instead of read out of two packet listings: where everyone stood and faced, each cast's area as
the client draws it, who hit whom for how much, knockbacks, deaths and the player's `!mark`s.

* **Sources**: either side is a battle run (Kawari Recorder's, from Diagnostics' battles, or one uploaded here: `questkit
  chronofoil --battle N.V --out run.json.gz` makes one from a Chronofoil capture; kept in `<web>/diag/replay-uploads`,
  the newest 100, numbers only) or one zone visit of a Kawari packet capture (`<data>/captures/*.jsonl`, cut at each
  ZoneInit; the visit with the most hits is picked first). Two retail runs or two Kawari tries can be compared too.
  `?a=<source>&b=<source>` in the address picks them (`upload:<file>`, `battle:<file>`, `capture:<file>:<visit>`),
  so a comparison can be linked.
* **Reading a capture**: from Kawari's `decoded` text, packet by packet: SpawnNpc/SpawnPlayer (kind, BNpcBase, BNpcName,
  layout, battalion, level, HP, position, facing), ActorMove and ActorSetPos, the player's own UpdatePositionHandler(
  Instance), HP from UpdateHpMpTp and EffectResult(Basic) (the packet's own actor's), ActorCast (action, target, cast
  time, where and which way), CancelCast, ActionEffect1/AoeEffectN (each target's damage, heal, miss and Knockback
  row), Kill, DeleteActor, DirectorVars (the director's sequence) and the `!mark` notes. A battle run is read from its
  Actors, Samples (`P`, `R`, `Hp`), Actions (effect kinds 3/5/6 damage, 4 heal, 1 miss, 31 knockback) and Events
  (casts from their text). One replay format for both (format 1, described in `battle_replay.py`).
* **Areas**: from `action-index.json` (*Read the action list from the game*, `kawari-questdump actions`), shaped as the
  world server's `ActionInfo::area_shape` (CastType 2/5/7 circles, 3/13 cones with the omen's `fanNNN` angle, 4/8/12
  lines and rectangles EffectRange long and XAxisModifier wide, 10 donuts with the omen's `sircle_AABB` hole, 11
  crosses), drawn at the caster (its position at the cast's start, its cast's facing), its target (followed until the
  cast ends) or the spot aimed at (CastType 7), filling as the cast goes, gone when it ends or is cancelled. An instant
  area (no cast before it) flashes at its source as it hits. Without the index, casts are dots with their action IDs.
* **Hits**: a line from the source to each target, red for damage, green for heals, with the amount; a knockback is an
  arrow away from the source, the Knockback row's distance long.
* **The clock**: play (¼× to 8×), the scrubber, arrow keys (shift: 5 s), space. Each side has its own 0 s ("This side's
  0 s is its N s"), set by hand or to its first hostile hit or first cast, so retail's and Kawari's fights line up
  however long the zone-in took. Under each map, a strip of that side's casts, hits, deaths, director steps and marks
  (click to jump), and a log of everything within 12 s, the moment's rows highlighted (click one to go to it).
* **The map**: framed on where the hits happened (*Frame everyone* for every position seen), one view for both sides
  when they're the same zone (*Same map*), drag to pan, wheel to zoom, a 5-yalm grid (25 at a distance). Hovering an
  actor shows its kind, BNpcBase, layout, BNpcName ID, battalion, HP and position. Sides are from battalions: yours and
  your battalion's are allies, every other battalion an enemy (Lominsan Way's 1771 is battalion 5, hostile to both).
* Actors show by numbers (BNpcBase and level), since there's no BNpcName list read from the game yet; the actions are
  named.
* Checked against Bob's bundle 20261104-204423 (The Lominsan Way's quest battle: Rive's line at the player, the hit and
  its knockback) and the retail run of Ul'dah's Most Wanted (Chronofoil a763f0a0), in a test ArchonEngine ControlPanel
  with an action index built from exported sheets; not yet on Bob's server with its own game install.

## Diagnostics (Dashboard → Diagnostics)

**Marks** (2.25.0): the bundle's README lists every `!mark` in its captures (time, file, line, what was seen), under the
packet captures ([Recorder and diagnostics](recorder-and-diagnostics.md#mark)).

**The quest summary** (2.24.1): `characters.json`'s `quest_summary.completed_ids` reads the completed quests in the
server's bit order (quest N is bit `0x80 >> (N % 8)` of byte `N // 8`, the world server's
`QuestBitmaskTransformation`). Before, it read `1 << bit` and listed the wrong quests (Bob's bundle 20261028-220355
named 682 and 685 by chance, but not 343, 511 or 513, which he had done).

**Who the bundle is about** (2.22.5): the README lists the chosen characters with whoever the window's captures were
played by first (the content ID in each capture's PlayerSetup), marked "played in this window", and
`characters.json` has `played_in_window`; before, they were in the database's order, so Bob's bundles read as Lira
Ormont's while he played Y'kolia Vhuh.

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
