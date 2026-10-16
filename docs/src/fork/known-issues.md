# Known issues and gaps

What is known to be missing, approximated or wrong in the fork. Remove an entry in the same commit that fixes it, and
add one when something is found.

## Quests

* **Close to Home** is only scripted for Conjurers (`ManFst004_00124`); the Lancer (`ManFst002_00085`) and Archer
  (`ManFst003_00123`) versions still stop after accepting.
* **The Gridanian Envoy** (507) and **Call of the Sea** (674) were written from Sapphire and the recording without the
  client's script: dump them in the Quest Lab to check the scene numbers. Envoy's arrival positions are Sapphire's.
* Story quests stop after **Call of the Sea** (674), which hands over to Limsa Lominsa.
* Limsa Lominsa and Ul'dah have only upstream's openings.
* A quest script without `onEnterTerritory` leaves an enter-territory event unfinished (the client waits). The fix (`02e22b5`) was rolled back with the first quest engine and not redone.
* Quest sequence/variable changes aren't logged or noted in packet captures any more (also lost in the rollback); use the captures' `UpdateQuest` packets or `//gm quest inspect`.
* `questkit replay` checks variables, notices, sequences and that events finish, not scene numbers: a scene the
  scaffolder guessed (marked TODO) needs the listing or a test in game.
* The quest battle prompt is always allowed (`ServerRequestReply` result 0); retail presumably checks something.

## Quest battles and directors

* **Battle talk**: Leia's Legacy and To Guard a Guardian use the window (DirectorPopupMessage) with retail's text IDs;
  Chasing Shadows and Spirithold Broken still say their lines in `/say` until their text IDs are recorded.
* Chasing Shadows' enemies only auto-attack (no skills yet).
* NPCs fighting each other use their stat-based damage (no retail recording of it), so allies with retail's low health
  could die: in Leia's Legacy and To Guard a Guardian every enemy goes for the player instead of the allies as on
  retail.
* To Guard a Guardian: which six Ixali come first is a guess from where they stand; after the cutscene retail moves the
  player through a zone change (PrepareZoning to the same territory), Kawari leaves them in place.
* The new quest battles' timelines (skills and how often) are rough loops from the recording's counts; only Janremi,
  the blanga and the lesser gargoyle have recorded damage to players.
* Retail sends the player out of a lost battle; the delays are copied from recordings, not from known rules.

## Combat

* **Enemy damage** without a `damage_to_players` recording falls back to 12 % of a player-style calculation; retail's
  enemy formula is unknown. Only Central Shroud monsters have recordings.
* **Area attacks** on several players show only the main target in the ActionEffect packet (only the one-target
  opcode is known); the others get flying text and HP updates.
* **Drop rates** are a 50 % guess except for anoles, ground squirrels and trickster imps; drops cover the Central
  Shroud only.
* **Aggro range** (15 yalms) and **leash distance** (40 yalms) are global constants; the Recorder's World tab can give
  per-enemy values.
* Player **HP** still uses upstream's approximate formula (low for levels 1–50).
* Most jobs besides White Mage and Dancer have only their starter skills. Cooldowns aren't sent
  (`server/action.rs` TODO), so the client doesn't show recasts from the server.

## Character windows

* **Portraits** still need the crash described (what, when, the client's crash log); the server now saves and returns
  the adventurer plate's portrait, but a character without one gets the same zeroes as before.
* **Achievements**: only Types 2, 3, 6 and 9 are completed automatically, and what those Types mean is the community's
  reading of the sheet, not checked on retail. Nearly completed lists are always empty. Whether retail also sends
  LogMessage 952 ("… earned the achievement …") on completion, or the client prints it, isn't known.
* **Recommendations** are worked out by the client; what's wrong on Kawari isn't known yet.

## Systems (upstream gaps the fork hasn't touched)

Crafting is a stub; gathering uses placeholder EXP; duty completion isn't saved per player; housing doesn't persist
furniture; market board, trade, retainers, blacklist and fellowships are placeholders. See `FORK_ROADMAP.md`.

## Kawari Web

* Gil purchases only work while the character is logged out (by design, see [Saving](saving.md)).
* 162 costume sets in the Shop have no contents yet and can't be bought until an admin adds them.
* Store items arrive as items; what they do when used depends on server support.

## Build and tests

* `inventory::equipped::tests::test_item_level` needs a real game install, so `cargo test --workspace` fails without
  one.
* Running `cargo fmt` on the workspace reformats upstream files; only check the files you changed
  (`rustfmt --check --edition 2024 <files>`).
