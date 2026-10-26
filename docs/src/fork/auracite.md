# Auracite Backups

[Auracite](https://github.com/redstrate/Auracite) is redstrate's Dalamud plugin that archives a retail character as
`character.json` in a zip. Kawari imports it two ways: the world server's own importer (`servers/world/src/auracite.rs`,
the lobby's import, which creates a new character) and Kawari Web's (`kawari_core/auracite.py`, which updates an
existing one; see [Kawari Web](kawari-web.md#auracite-backups-2220)). Kawari Web also exports a character back out.

## The extended plugin

**Auracite (Kawari)** is the fork's extension of the plugin, in the private kawari-diagnostics repository
(`auracite/`, with its `SCHEMA.md`). It writes everything upstream Auracite 1.8.1.3 writes, unchanged, so upstream's
importer still reads it, plus a top-level `kawari_extended` object: crystals, key items, both saddlebags, the quests in
progress with their variables, levequests, daily quests, legacy quests, map markers, the gathering and crafting logs,
tribal reputation, the glamour dresser, plates and armoire, the chocobo companion, gear sets, retainers, achievements,
logs and records, hotbars, macros, and raw copies of the client structures it reads. Everything is optional, and a
section the client only loads once its window has been opened (the dresser, the armoire, the saddlebag, achievements,
a retainer's items) says `*_loaded: false` when it wasn't: importers leave what the character has alone rather than
clearing it.

## What each importer takes

Both importers read the same members (the server's `apply_extended`, Kawari Web's `build_changes`):

| `kawari_extended` | Kawari |
|---|---|
| `crystals`, `key_items` | inventory (crystals keep their slot's item ID when empty) |
| `saddlebag` (if loaded) | `inventory.saddlebag`, both pages (Kawari Web makes them first on an inventory saved before Kawari had a saddlebag: from 2.22.3; before, it dropped the saddlebag there) |
| `glamour.dresser`, `.plates`, `.armoire` (each if loaded) | `inventory.glamour` ([Glamour](glamour.md)) |
| `active_quests` | `quest.active`: ID less 65536, sequence, the six variables (flags and the accepting class aren't kept) |
| `completed_leves`, `completed_legacy_quests`, `unlocked_map_markers`, `gathering_log` | the `quest` table's bitmasks |
| `adventurer_plate` (if loaded) | `volatile.adventurer_plate` (CharaCardData, 188 bytes), `adventurer_plate_saved` (its timestamp) and `volatile.portrait` (BannerData, 52 bytes): the client's own CharaCard, the same bytes it sends when it saves the plate (UpdateAdventurerPlate, SetAdventurerPlatePortrait) |
| `companion` | the `buddy` row: name, rank, stars, EXP, skill points, stance (BuddyAction 3–7 only), favourite feed, colour, barding, skill lines ([Chocobo Companion](chocobo.md)) |

Kawari Web puts the inventory members under *Inventory*, the quest members under *Quest progress*, and the companion
under a new *Chocobo companion* section (also allowed for Character sync, off by default).

**Everything is kept.** The whole `kawari_extended` object is stored with the character in world.db's
`auracite_extended` table (`content_id`, `data` as JSON text, `imported_at`; migration
`update-2026-10-24_auracite_extended`), by both importers, replacing the one from an earlier import. Kawari Web's
export writes it back as `kawari_extended`, with Kawari's current data for the members above (they may have changed
in game since), so gear sets, hotbars, macros, achievements, retainers, logs and the raw blocks survive a Kawari →
Auracite → Kawari round trip. The server deletes the row with the character.

## Gear sets, hotbars, macros and the UI

These never reach a server: the client keeps them in the game's user folder on the PC, in `FFXIV_CHR<content ID in
16 hex digits>` (GEARSET.DAT, HOTBAR.DAT, MACRO.DAT, KEYBIND.DAT, ADDON.DAT, UISAVE.DAT…). The same character on Kawari
has another content ID, so its folder starts empty: that's why an imported character had no gear sets (Bob,
2026-10-24). The extended backup carries the folder's files (`config_files`: every top-level .DAT but ACQ.DAT, base64),
which the importers keep, and the plugin's window restores them into another character's folder: armed with
*Restore when I log out* while logged in to that character, it writes them a few seconds after the logout (the client
writes its own files as it logs out), moving what was there to a `before-restore-<date>` folder; at the character
select screen it restores at once. Log back in and the gear sets, hotbars, macros and layout are retail's.

## Not done

See [Known Issues](known-issues.md#auracite-backups).
