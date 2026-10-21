# Party Bots

Player-like NPCs that join you, follow your party's tank and fight by their class's role, so a dungeon can be run
without other players. Delivered in stages; this page describes what exists and what's planned.

| Stage | What | Status |
|---|---|---|
| 1 | `!spawn playernpc <class>`: a bot of any class joins you anywhere, follows, and fights by role | done (2026-10-21), untested in game |
| 2 | The Duty Finder fills the roles your party lacks with bots (a tank, a healer, two DPS), level synced to the duty; unsynced duties get none; bots in the party list as retail lists Duty Support members | needs a retail Chronofoil capture of a Duty Support run (Sastasha) for the party list packets |
| 3 | Dodging area attacks, killing adds first, following each dungeon's rules (Sastasha first) | to do |

## Spawning one

`!spawn playernpc <class>` in Say (any class's or job's name, without spaces or not, or its abbreviation, in the
server's language or English: `whitemage`, `White Mage`, `WHM`, `paladin`, `ARC`). The bot appears beside you and its
name and class are printed. `!spawn playernpc dismiss` sends all of yours away. (It's a `!` debug command, not `//gm`:
the client parses `//gm` itself and only sends commands it knows, and it has no `spawn`; see
[Tools and Debug Commands](tools.md).)

Code: `servers/world/src/server/party_bot.rs` (`spawn_party_bot`, `update`), the command in `server/chat.rs`.

## What a bot is

* **An NPC with a player's look.** A battle NPC of BattleNpcSubKind 9, `NpcPartyMember` (retail's Duty Support
  members are this kind), battalion 0 (your side), ModelChara 0 (a human model built from its customize data and gear).
  Player area attacks don't hit it (they only hit `Enemy` NPCs).
* **A random look.** One of the first six races (Hyur, Elezen, Lalafell, Miqo'te, Roegadyn, Au Ra) and either sex, its
  face and hairstyle 1–4 (which every tribe and sex has) and random features and colours within small ranges. Hrothgar
  and Viera are left out until their valid ranges are read from the character creator's data.
* **A made-up name:** two words built from syllables (not the game's text), within retail's 20 characters.
* **Level 100** (`BOT_LEVEL`) with **the best gear its class can wear at that level**, found in the Item sheet at
  runtime (`GameData::best_gear`): for each slot (weapon or two-handed weapon, off-hand, head, body, hands, legs, feet,
  ears, neck, wrists, two rings) the item of the highest item level whose ClassJobCategory includes the class and
  whose LevelEquip is at most the bot's level, rarity then the newest breaking ties; PvP gear left out; no off-hand with
  a two-handed weapon; two of a ring that isn't unique, otherwise the best two.
* **A player's stats.** The same formulas as players' (`use_retail_formula`, `perform_calculations`, the gear's stats
  as high quality, `calculate_potencies`): its class's modifiers, its tribe, level 100 and that gear, so its HP, damage
  and healing are a player's. 10,000 MP.
* **Its class's actions.** Every player action of the class (or its base class, for a job) learned by its level
  (Action sheet: IsPlayerAction, ClassJob, ClassJobLevel; no PvP or role actions) that Kawari has an action script for.
  Kawari's scripts cover levels 1–50 of the base classes and White Mage (see [Combat](combat.md)), so a level 100 bot
  fights with those for now; once synced to a level 1–50 duty that's its whole kit anyway.

## How it fights

Every server tick (500 ms), before NPCs move, `party_bot::update` tells each bot what to do:

* **Out of combat** it follows the party's tank (you, if you're on a tank class; else a tank bot), or you, keeping
  2.5 yalms behind; the tank bot follows you. It regenerates 10 % of its HP every 3 s between fights.
* **In combat**: anything with you or a bot on its enmity list is fighting the party.
  * The **tank** goes for whatever is after someone else (the nearest first) and pulls it onto itself every 10 s
    (50,000 enmity above the top of that enemy's list, as the quest battle framework's tanks do), otherwise keeps on
    what it's fighting.
  * **DPS** attack what the tank is attacking (the enemy most after a tank bot, or your target if you're the tank),
    else your target, else what they were on.
  * The **healer** heals whoever in the party is lowest when someone is below 60 % (its highest level single-target
    heal on the GCD, at most once a GCD), and otherwise attacks like a DPS.
  * Melee bots fight from 3 yalms, ranged bots and healers from 15 and stay within 20 of the one they follow.
* **Its rotation** (`build_rotation`, a heuristic until bots' rotations are recorded), run by the NPC class rotations
  (`server/rotation.rs`, a 2.5 s pace like a player's GCD): its abilities whenever they're ready; its combos, each step
  after the one it follows (highest level first, so a combo runs to its end); its damage over time spells and
  weaponskills re-applied 2 s before they wear off (the duration in their script); and its filler: the highest level
  combo starter, or its strongest plain weaponskill or spell (the potency in its script), so Stone II replaces Stone.
  Single-target attacks only. Melee bots auto-attack (870), physical ranged ones too (872); casters don't.

## Not done yet

* Bots aren't in the party list, don't follow you into another zone (they leave when you do), aren't level synced,
  and stay down when they die (dismiss and spawn again).
* No area attacks, buffs, role actions, mitigation, Raise, or dodging enemy area attacks.
* Targets are picked by these simple rules, not by watching retail Duty Support members; their rotations, heal
  thresholds and pull timing are guesses to be checked against a recording.
* Nothing has been tried in game yet: the look, the gear models and how the client shows an `NpcPartyMember` outside
  a duty are the first things to check.
