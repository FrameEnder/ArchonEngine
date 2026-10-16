# Known issues and gaps

What is known to be missing, approximated or wrong in the fork. Remove an entry in the same commit that fixes it, and
add one when something is found.

## Quests

* **Close to Home** is only scripted for Conjurers (`ManFst004_00124`); the Lancer (`ManFst002_00085`) and Archer
  (`ManFst003_00123`) versions still stop after accepting.
* Story quests stop after **Salvaging the Scene** (387). The next ones need a new retail recording.
* Limsa Lominsa and Ul'dah have only upstream's openings.
* A quest script without `onEnterTerritory` leaves an enter-territory event unfinished (the client waits). The fix (`02e22b5`) was rolled back with the first quest engine and not redone.
* Quest sequence/variable changes aren't logged or noted in packet captures any more (also lost in the rollback); use the captures' `UpdateQuest` packets or `//gm quest inspect`.
* The quest battle prompt is always allowed (`ServerRequestReply` result 0); retail presumably checks something.

## Quest battles and directors

* **Battle talk** (the pop-up window) packet isn't known for this client; lines are said in `/say` instead.
* Chasing Shadows' enemies only auto-attack (no skills yet).
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
