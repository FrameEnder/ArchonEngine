# Party Bots

Player-like NPCs that join you, follow your party's tank and fight by their class's role, so a dungeon can be run
without other players. Delivered in stages; this page describes what exists and what's planned.

| Stage | What | Status |
|---|---|---|
| 1 | `!spawn playernpc <class>`: a bot of any class joins you anywhere, follows, and fights by role | done (2026-10-21), untested in game |
| 2 | The Duty Finder fills the roles your party lacks with bots (a tank, a healer, two DPS), level synced to the duty; unsynced duties get none; bots in the party list | done (2026-10-22), untested in game: the party list as retail lists a Duty Finder party (Bob's capture of five Sastasha runs; no Duty Support run recorded) |
| 3 | Dodging area attacks, killing adds first, following each dungeon's rules (Sastasha first) | to do |

## The Duty Finder (stage 2)

Entering a duty **alone** through the Duty Finder (or `!cf <id>`) fills the party with bots once you've loaded in
(`party_bot::fill_duty_party`, called from the zone-loaded handler; `JoinContent` marks the player in
`Instance::pending_duty_bots`):

* **Only level-synced duties of one party** (ContentFinderCondition's ClassJobLevelSync, and its ContentMemberType of
  one party of 2–8: `GameData::content_party_roles`). Unsynced duties, alliance raids and solo duties get no bots.
* **The roles you don't play**: the duty's tanks, healers, melee and ranged DPS (ContentMemberType), less your own
  place (a DPS takes a place of their own kind first). Sastasha: you as a healer get a tank and two DPS.
* **Each bot is a random base class of its role** (Gladiator or Marauder; Conjurer; Pugilist, Lancer or Rogue;
  Archer, Thaumaturge or Arcanist: the classes Kawari has scripts for), **at the duty's synced level**, with the best
  gear its class could wear at that level and a player's stats for it.
* **The party list** lists you first, as leader, then the bots: each with its actor ID, class, level, HP and MP, the
  zone, `sync_positions` 1, no entity or parent ID, then a PartyUpdate JoinParty, as retail's Duty Finder parties were
  listed (Bob's Chronofoil capture of 2026-10-22: five Sastasha runs; the PartyList as the duty's ZoneInit arrives,
  4 members, 456 bytes each). The bots' account and content IDs are made up from their actor IDs, and the party ID too
  (`0x00B1…` and the player's actor ID); the server keeps no party for it, so your `party_id` stays 0 and party chat
  and invites work as before. Leaving the duty dismisses the bots and sends an empty list with DisbandingParty, as
  retail does at the end.
* A party of real players goes in as before, with no bots.

What retail's Duty Finder parties showed (numbers only, the same capture): every member's level 18 in the list
(Sastasha's enemies are 15), HP 351–652; the list's order is the order members joined (you first when you were
first, last otherwise); PartyMemberPositions every 5 s; when a member leaves, a new list and PartyUpdate
MemberLeftParty (6) to the rest; at the end DisbandingParty (3) with an empty list, then an empty one again in the
city. No Duty Support run is recorded yet, so how retail lists Duty Support members is still unknown.

## Bots and battle NPCs are kept apart

Everything bots do is keyed to the bots themselves (`Instance::party_bots`, the actors `spawn_party_bot` made), never
to a kind of NPC, so quest battle allies, enemies and other battle NPCs fight as before, from their scripts, drop-ins
and timelines, as retail's recordings show: `party_bot::update` only gives orders to `party_bots`; walking straight
where there's no navmesh path and regenerating between fights check `instance.party_bots` too; bots follow because
they're owned (`owner_id`, as pets are), not because of their kind.

## Spawning one

`!spawn playernpc <class>` in Say (any class's or job's name, without spaces or not, or its abbreviation, in the
server's language or English: `whitemage`, `White Mage`, `WHM`, `paladin`, `ARC`). The bot appears beside you and its
name and class are printed. `!spawn playernpc dismiss` sends all of yours away. (It's a `!` debug command, not `//gm`:
the client parses `//gm` itself and only sends commands it knows, and it has no `spawn`; see
[Tools and Debug Commands](tools.md).)

Code: `servers/world/src/server/party_bot.rs` (`spawn_party_bot`, `update`), the command in `server/chat.rs`.

## What a bot is

* **An NPC with a player's look**, spawned as retail spawns a **chocobo companion**: a battle NPC of sub-kind 3
  (`Chocobo`), owned by the player (`owner_id`), its name in the spawn's text and no BNpcName (`name_id` 0), character
  data flags 0x1, no display flags, battalion 0 (your side), ModelChara 0 (a human model built from its customize data
  and gear). Its nameplate then shows its name (Bob, in game, 2026-10-21). Retail's party-member NPCs (Trust, six in
  Bob's Chronofoil captures in territories 822 and 884) are sub-kind 9 (`NpcPartyMember`), owned, display flags
  0x40000, character data flags 0x2 and a BNpcName; spawned that way (`kind=partymember`) a bot's nameplate shows
  only "Lv100", since for that kind the client takes the name from the BNpcName. Player area attacks don't hit
  bots (they only hit `Enemy` NPCs).
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

* **Movement.** Where the navmesh has no path (none around Summerford Farms' houses), a bot walks straight to where
  it's going, as NPCs in content do; out in the world other NPCs give up, and bots stood still while you walked off
  (Bob's 2026-10-21 bundle: 550 failed paths, both ends off the mesh).
* **Out of combat** it follows the party's tank (you, if you're on a tank class; else a tank bot), or you; the tank bot
  follows you. Each bot has **its own spot in a formation** behind the one it follows (`FORMATION`, by the order they
  joined: 35° either side of straight behind at 2.5 yalms, straight behind at 3.8, out to the sides at 75°, then
  further back; more than seven start another ring 3.5 yalms out), turned with the way they face. It walks to its spot
  when it's more than 0.6 yalms off and stops there. Out of combat **the bot walks itself** (`PartyBot::walking_itself`):
  `party_bot::update` keeps its path, moves it along it at NPCs' running speed (6 yalms a second, 3 a tick) and sends
  the ActorMove, and NPCs' own walking (`npc_behavior`) leaves it alone until it fights, when that takes over to chase
  its target. While you move it **walks with you**: its path is re-aimed at where its spot is now, from where it
  stands (the navmesh, straight on where that stops), at most once a second (`REAIM_EVERY`) and only when the spot has
  moved a yalm from the path's end. (The first build had every bot walk to within 2.5 yalms of the same person, so
  they ended up on top of each other; the next threw its path away each time the spot moved a yalm, which is every
  tick while you run, so bots only set off once you stopped; the one after left both its own code and NPCs' walking
  steering it, and NPCs' walking pulled it back along its old path, so it went to and fro at its spot: Bob's
  2026-10-21 bundles and testing.) It regenerates 10 % of its HP every 3 s between fights.
* **In combat**: anything with you or a bot on its enmity list is fighting the party.
  * The **tank** plays as the player tanks in Bob's Chronofoil capture of five Sastasha runs (2026-10-22) did; the
    three runs by other players (Paladin, Marauder, Warrior) give the numbers:
    * It answers each enemy that's after someone else after one of those tanks' reaction times (`TANK_REACTIONS`: 54
      of them, 2.9 s the median, from an enemy hitting someone else to the tank's next action hitting it), the nearest
      due first: it goes for it and takes it (50,000 enmity above the top of its list, as the quest battle framework's
      tanks do). It used to take one every 10 s.
    * With two or more enemies fighting the party within 5.5 yalms, its area enmity combo (Total Eclipse, Overpower:
      `tank_aoe_action`) is its GCD, and keeps each enemy it reaches on it (`TANK_STANCE_ENMITY` above the rest); those
      tanks made it their main attack then (Total Eclipse 74 times to 38 single-target hits, Overpower 62 to 43 and
      117 to 37).
    * An enemy it hasn't hit yet, 6–20 yalms away, gets its ranged attack (Shield Lob, Tomahawk: `tank_pull_action`),
      as those tanks threw it from 6–21 yalms (14 the median) at enemies joining the fight.
    * An enemy that turns on someone else counts as fresh again, so the tank throws its ranged attack at it when it's
      6–20 yalms off; further, it runs at it until it's in range.
    * It stands **across its target from the rest of the party** (`Placement::opposite`, 2.5 yalms beyond it), so the
      enemy faces away from them and its cones and cleaves miss them. It moves round only when the party is less than
      120° round from it, seen from the enemy (the capture's tanks were past that about half the time: 113° the median
      over 313 hits).
    * It **keeps up with the player** (unless the player is the tank): more than 15 yalms from them while fighting (the
      player has run on ahead, pulling as they go, wall to wall), it goes after them, and keeps throwing its ranged
      attack at whatever turns on someone and using its area combo on what comes along with it.
    * Otherwise it keeps on what it's fighting; its rotation fills the rest (it's told about the area attacks and
      pulls, so it doesn't use another GCD at once).
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

## Glamours

Each bot can wear its own outfit over its gear: one picked at random, for its sex and class, from a list the server's
owner writes (`servers/world/src/server/glamour.rs`). The Chrome extension in `tools/glamour-collector` builds the list
from Eorzea Collection's glamour pages ([Tools](tools.md#glamour-collector-toolsglamour-collector)). The list is **`glamours.txt` in the server's data folder**, beside
`config.yaml` (Kawari Web's `KAWARI_DATA_DIR`, `~/.kawari` unless changed). It's kept out of git, as the item names in
it are the game's. The server reads it the first time a bot joins and again whenever it changes, and logs how many
glamours it read and any item or dye it couldn't find.

```text
# One glamour per block. A line starting "==" names it; end it with (male) or (female) for the sex it's for, or
# (ANY), (either) or neither for either. An optional Class line lists the classes or jobs it's for. Then one line per
# piece: the item's name as the game spells it, then | and its dyes.
== Seaside captain (female)
Class: <Black Mage> | <SCH>
Head: <item name>
Body: <item name> | <dye>
Hands: <item name> | <dye> | <second dye>
Legs: <item name>
Feet: <item name> | <dye>
```

* The slot before the colon is optional (the item says where it goes); weapon lines are skipped (bots keep their
  class's weapon). Names match whatever the case or spacing, and either kind of apostrophe, with or without the
  `<...>` round them (the extension writes them); `undyed` means no dye.
* `Class:` (or `Job:`) takes classes and jobs by name, English name or abbreviation (`Black Mage`, `BLM`), `|`
  between them; several lines add up. A job and its base class count as one: a Black Mage glamour fits a Thaumaturge
  bot and a Thaumaturge glamour fits a Black Mage, but jobs sharing a base class don't fit each other (a Scholar
  glamour fits an Arcanist, not a Summoner). Without the line a glamour fits any class; one whose classes are all
  unknown is left out (and logged) rather than worn by every class.
* Names are looked up in the game's Item sheet (equippable items), Stain sheet and ClassJob sheet (combat classes), in
  the server's language.
* A piece goes in the slots its EquipSlotCategory fills, as the gear's glamour, and empties the slots it covers (a
  long coat over the legs). Only the looks change; the bot's stats are its gear's.
* Without the file, bots wear their gear's looks, as before.

## Not done yet

* In combat melee bots still crowd the same spot by their target (only following is in formation).
* Bots don't follow you into another zone (they leave when you do) and stay down when they die (dismiss and spawn
  again). Bots from `!spawn playernpc` aren't in the party list or level synced (only the Duty Finder's are).
* The party list doesn't follow the bots' HP (retail sends HP through the members' actors, which the client may or may
  not take from an NPC), and PartyMemberPositions isn't sent for bots (no minimap dots).
* Bots don't know a dungeon's route: the tank bot follows you, so you lead.
* Bots always pass on coffers' loot (see [Coffers and Loot](loot.md#party-bots)) and don't open coffers.
* The tank doesn't Provoke (a role action, not in bots' kits), use mitigation, or pull packs before the player.
* No area attacks, buffs, role actions, mitigation, Raise, or dodging enemy area attacks.
* Targets are picked by these simple rules, not by watching retail Duty Support members; their rotations, heal
  thresholds and pull timing are guesses to be checked against a recording.
* Nothing has been tried in game yet: the look, the gear models and how the client shows an `NpcPartyMember` outside
  a duty are the first things to check.
