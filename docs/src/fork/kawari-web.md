# Kawari Web

Kawari Web (`FrameEnder/Kawari-Web`, Flask, currently **2.12.0**) runs beside the server in its own container. Its
README is the user guide (setup, pages, service, forges, builds); this page is about **how its features work** and
how they depend on the fork. Code: `kawari_core/` (logic, no Flask) and `kawari_web/` (blueprints, templates).

## What it touches on the server

| Data | Read/write | Used by |
|---|---|---|
| `login.db` | read/write | accounts (one account for the site and the game launcher), password changes |
| `world.db` | read; writes only for logged-out characters (and `mail`) | profiles, Progress, Quest Lab character setup, Shop gil and delivery |
| `config.yaml` | read/write | Dashboard config editor, Diagnostics toggling `world.packet_capture` |
| The active server image | runs throwaway containers | `kawari-questdump` (Quest Lab, Progress index, Shop item list) |
| `<data>/quest-dumps/` | written by questdump | `quest-index.json`, `item-index.json`, per-quest dumps |
| `<data>/captures/` | read | Diagnostics bundles |
| The web app's own SQLite db | read/write | feed, notifications, settings, shop tables |

The rule from [Saving](saving.md#editing-a-character-from-outside) applies: the server saves a logged-in character
over the database, so direct edits only happen while the character is logged out. Mail is safe at any time.

## Pages (summary)

Home, How to Connect, Feed and profiles (character tile boards, private by default), Notifications, Settings,
**Shop**, and the admin **Dashboard**: server control, config editor, database health/migrations, backups, accounts,
Auracite import/export, branding, Quest Lab, Progress, Diagnostics, Updates (which fork/forge/commit the server is
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

## Quest Lab (Dashboard → Quests)

Runs `kawari-questdump` for chains, single quests, raw files and the index; shows each quest's variables, objectives,
listeners and rewards beside the fork script and the client listing; **script check** (variables the Quest sheet
doesn't have, scenes the client script doesn't contain, `-- NOTE: unverified` markers); copyable `//gm quest`
commands; **Set up a character** (logged out, snapshot first: completes prerequisites, taking one branch of "any one
of" — the quest's own city — raises the class to the quest level, leaves the quest ready, at a sequence, completed or
never started); a zip bundle of dumps, scripts and `Global.lua` with a manifest. Dumps stay private.

## Quest progress (Dashboard → Progress)

Every quest from A Realm Reborn to Dawntrail from the quest index (format 2), Main Scenario when in ScenarioTree,
side quests by journal section. **Implemented** = the active build has
`resources/scripts/events/quest/<id/100>/<script>.lua`; **verified** = its header says it was checked against a retail
recording. Totals per expansion and a filterable list linking to Quest Lab.

## Diagnostics (Dashboard → Diagnostics)

Commits one bundle to the private `kawari-diagnostics` repository (fine-grained token, Contents read/write): recent
server log, packet captures (toggle here), chosen characters' saved quest state and position, Kawari Probe snapshots
(`POST /api/diag/snapshot` with the character-sync token), build and redacted config. Can also point
`filesystem.navimesh_path` at `<data>/navimesh` so navmeshes survive rebuilds. Bundle format:
[Recorder and diagnostics](recorder-and-diagnostics.md).
