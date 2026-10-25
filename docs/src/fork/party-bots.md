# Party Bots

Player-like NPCs that join you, follow your party's tank and fight by their class's role, so a dungeon can be run
without other players. Delivered in stages; this page describes what exists and what's planned.

| Stage | What | Status |
|---|---|---|
| 1 | `!spawn playernpc <class>`: a bot of any class joins you anywhere, follows, and fights by role; shown as another player (2026-10-23) | done (2026-10-21), untested in game |
| 2 | The Duty Finder fills the roles your party lacks with bots (a tank, a healer, two DPS), level synced to the duty; unsynced duties get none; bots in the party list | done (2026-10-22), untested in game: the party list as retail lists a Duty Finder party (Bob's capture of five Sastasha runs; no Duty Support run recorded) |
| 3 | Dodging area attacks, killing adds first, following each dungeon's rules (Sastasha first) | to do |

## The Duty Finder (stage 2)

**Roulettes** (QueueRoulette) roll only among the roulette's duties the queuing character, who is the party's leader,
has unlocked (the dungeon, guildhest, trial and raid bitmasks `unlock_content` sets) and whose ClassJobLevelRequired
their current class meets (`ZoneConnection::pick_roulette_duty`, `GameData::roulette_duties`). Before Bob's bundle of
2026-10-24 it rolled any duty in the roulette. With none to roll, nothing is queued (logged).

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
city. Retail lists Duty Support members another way (ActorControlSelf per party slot, no PartyList): see
[Party NPCs](party-npcs.md#how-retail-spawned-and-listed-them).

## Bots and battle NPCs are kept apart

Everything bots do is keyed to the bots themselves (`Instance::party_bots`, the actors `spawn_party_bot` made), never
to a kind of NPC, so quest battle allies, enemies and other battle NPCs fight as before, from their scripts, drop-ins
and timelines, as retail's recordings show: `party_bot::update` only gives orders to `party_bots`; walking straight
where there's no navmesh path and regenerating between fights check `instance.party_bots` too; bots follow because
they're owned (`owner_id`, as pets are), not because of their kind.

## Spawning one

`!spawn playernpc <class>` in Say (any class's or job's name, without spaces or not, or its abbreviation, in the
server's language or English: `whitemage`, `White Mage`, `WHM`, `paladin`, `ARC`). The bot appears beside you and its
name and class are printed; it follows you, and joins your party when you invite it (below).
`!spawn playernpc dismiss` sends all of yours away. (It's a `!` debug command, not `//gm`: the client parses `//gm`
itself and only sends commands it knows, and it has no `spawn`; see [Tools and Debug Commands](tools.md).)

Bots are **shown as other players**: the standard since Bob saw it work in game (2026-10-23). The older looks stay as
a fallback:

* `kind=chocobo` or `kind=partymember` after the class shows that one bot the old way (see
  [What a bot is](#what-a-bot-is)): `!spawn playernpc PLD kind=chocobo`.
* `!spawn playernpc look <player|chocobo|partymember>` sets how bots are shown from then on, the Duty Finder's too,
  until the server restarts (`party_bot::set_default_look`); bots already there keep theirs.

Code: `servers/world/src/server/party_bot.rs` (`spawn_party_bot`, `update`), the command in `server/chat.rs`.

## When a bot dies

A dead bot **lies where it fell**, as a player does: `kill_actor` no longer fades it out and despawns it after 8 s as
it does every other NPC (before, it was gone before anyone could raise it). Then (`party_bot::tend_the_fallen`):

* **Raised** (anyone's Raise, Resurrection and the like, which offer the dead a revive, status 148): it accepts after
  2 s and gets up there as a raised player does (the server's `revive_in_place`): 20 % of its HP and MP, Weakness
  (43, 100 s; Brink of Death, 44, if already weakened) and status 418 (5 s), and for everyone near, the raise's
  stand-up (ActorControl ZoneIn with its raise animation, as others see a raised player, then its mode back to
  normal).
* **Not raised within 30 s**: it gets up by itself beside the one it follows, at full HP and MP, and carries on
  (spawned again there: clients forget it where it lay). Unlike a player returning to their home point, it gets no
  Weakness.

**Healer bots raise** (`party_bot::raise_the_fallen`): a healer bot whose class has learnt a raise Kawari has a script
for (Raise 125 for Conjurer and White Mage at 12; Resurrection 173, Ascend 3603 and Egeiro 24287 by ID, for when those
classes are bots) casts it, in combat or out of it and before healing, on the nearest dead member of its party (the
player it follows, or that player's other bots) within the raise's range (the Action sheet's, 30 yalms) that hasn't a
revive offered already and isn't being raised by another bot. A player accepts it as from anyone; a bot accepts by
itself. The bot stands still while it casts (Raise's 8 s), out of combat too.

**Keeping to the ground.** Where the navmesh has no path, a bot walks straight on (so it isn't left behind), and a
straight line between two points went under a rise in the floor: in Bob's Sastasha bundle of 2026-10-23 the tank
walked 2 yalms under the ground the player had just walked, its height falling evenly from 45.79 to 45.73 while the
floor rose to 47.8. Each step of a bot's (walking itself out of combat, and NPC walking in combat) is now set on the
navmesh's floor right under it, where there's one within 3 yalms (`Navmesh::ground_under`).

## Bots kept as friends

A bot sent a friend request is **kept**: `party-bot-friends.json` in the server's data folder (beside `world.db`,
`/kawari/data` in Kawari Web's container; kept out of git), by your content ID: its made-up content ID, name, class,
look (its 26 customize bytes as hex) and glamour (by name), and when (`server/bot_friends.rs`). A bot isn't a
character, so there's no row for it in the database's friend table:

* **The friend list** adds your kept bots after your other friends (`refresh_friend_list`), each online, in your
  zone, at level 100, with the context menu, so the client offers *Invite to Party* on them.
* **Invite to Party** on a kept bot that isn't with you **calls it**: it appears beside you, the same name, class,
  look and glamour (if the glamour list still has it; otherwise none), at the duty's synced level when you're in one
  (100 otherwise), already in the party, and the invite is accepted as for a bot that's there. Its content ID is the
  same each time (`PartyBot::content_id`), so the friend list's entry, the party list's and the character are one.
* **Remove from Friend List** forgets it (the file loses it; FriendRemoved to you).

## Party, and examining a bot

Bots are characters to the client, so its menus on them work (`party_bot::handle_bot_party_messages`):

* **Invite to Party** on one of your bots (from its menu, or the party finder's): it accepts at once, as a player
  would. You get the invite's result, then *invite accepted*, then the party list with it and a PartyUpdate
  JoinParty naming it (the leader executes, the bot is the target, as a real join is sent). `!spawn playernpc` bots
  start outside the party (`PartyBot::in_party`), the Duty Finder's inside. Refused (*That player is already in
  another party.*) while you're in a party of players (bots can't join one the server keeps), past eight, or when
  it's already in. How it gets there: no character has the invite's content ID, so `send_social_invite` hands it
  to the server as `ToServer::BotPartyAction` rather than giving up. The client's invite from a character's menu
  carries **only the name** (content ID 0, world 63; Bob's bundle of 2026-10-23, where the first version, which
  matched content IDs only, ignored it), so a bot is found by its made-up content ID or, with 0, by its name among
  the inviter's bots; a friend request carries the content ID.
* **Kick** a bot from the party list: it leaves and goes (dismissed); the list without it and MemberKicked naming it.
  The bot is found by the kick's content ID, or by its name (as the invite is) when the ID matches none.
* **Add to Friend List** on a bot: it's kept as your friend (below) and accepts at once (the invite's result, then
  *invite accepted*, of the pending-friend kind, as Kawari sends a player's acceptance).
* **Send Tell** to a bot: it answers by tell. `dismiss`: it goes (and leaves the party, MemberLeftParty). `leave`: it
  leaves the party and keeps following. Anything else: it lists these. The chat connection finds no character by
  the name, so it hands the tell to the server (`ToServer::BotTell`), which answers as the bot (its tell is sent
  from the bot's actor, with its made-up content ID), or with the usual *not found* when it's none of your bots.
* **Leave Party** or **Disband Party**: every bot in it goes, and the list empties (DisbandingParty).
* **Promote** a bot to leader: ignored (a bot can't lead).
* **Changing zone**: bots don't follow, so when they're dropped (`update`), a player who had them in the party gets
  an empty list.

A party of bots is none the server keeps (your `party_id` stays 0), and **Kawari's party code panicked on party 0**:
`PartyMemberKick` and `PartyChangeLeader` unwrap the party (*Why are we trying to do party operations on an invalid
party?*), so kicking a Duty Finder bot or making one leader would have stopped the server. With `party_id` 0 the
connection now sends these four to the bots' code instead.

**Examine** (the client's trigger 300, `ExamineCharacter`, with the target's actor ID) went nowhere: Kawari answered
every Examine with a blank window named "test". The server now answers (`ToServer::ExamineCharacter`), from the one
examined as retail does: for a bot its class, level, gear (each piece, the glamour over it, both dyes), average item
level, look, models and stats; for a player what their spawn shows (class, level, title, models, look; their items
and stats aren't on the server's side, so those are left empty). The layout (`party_bot::examine_bytes`), read from
13 ExamineCharacterInformation packets in Bob's Chronofoil captures (2026-09 and 2026-10, numbers only), each matched
with the same player's SpawnPlayer and the Item sheet:

| Offset | What |
|---|---|
| 0 | 4, then 1 (once 0), the class, the level |
| 6 | the title (u16) |
| 8, 9 | Grand Company, its rank |
| 10 | unknown (0, 2, 3, 5, 6) |
| 16–24 | 0, 0xFF×8 or a value (a free company?) |
| 32, 40 | the main and off-hand weapon models (u64, as in the spawn) |
| 50 | unknown u16 (57 in most; 78, 407) |
| 72 | the average item level (u16: 790 at level 100, 201 at 60, 7 at 23): the twelve gear slots' item levels each capped at ParamGrow[level].ItemLevelSync, a two-handed weapon counted twice, / 12, for the 6 examined players below level 100 (the cap lowered 4 of them: 215 → 201 at level 60); the 2 at level 100 weren't capped (790 and 775, above level 100's 690). Kawari's bots: `EquippedStorage::calculate_item_level`, uncapped (their gear is their level's); it counted a one-handed weapon without a shield twice before 2026-10-25 |
| 80 | 14 equipment slots of 40 bytes, in the equipment's order (main hand, off hand, head, body, hands, belt, legs, feet, ears, neck, wrists, right ring, left ring, soul crystal): the item (u32), its glamour's item (u32), the crafter's content ID (u64), 1 for high quality, a byte, five materia (u16 type, u16 grade: 16 and 11 in one), the two dyes |
| 640 | the name (32 bytes) |
| 672–704 | 0 in all 13 |
| 704 | the look (26 bytes, as in the spawn) |
| 732 | the ten gear models (u32 each) |
| 772 | their second dyes (10 bytes) |
| 782 | the two facewear IDs (u16 each: 460 in one) |
| 816 | the stats, as PlayerStats' first 30 (strength to perception: a level 60 Dancer's 199, 800, 783, 189, 174, 218, HP 8863, MP 10000, …) |

The packet is 944 bytes (Kawari's `ExamineCharacterInformation`, whose two unknown arrays are these two parts).

## What a bot is

* **Shown as another player** (since 2026-10-23, the standard; Bob saw them as players in game the same day). To the server a bot is the same
  owned battle NPC as below (sub-kind 3), so its fighting, following and everything keyed to `party_bots` is
  unchanged, and enemies, pets and area attacks treat it as before; only what clients are sent differs. The actor
  holds `shown_as_player` (`ShownAsPlayer`: its account and content ID, the leader's world and content handler), and
  `NetworkState::spawn_existing_actor_message`, the one place an actor's spawn is made for a client, sends a
  **SpawnPlayer** built from its NPC spawn as it is then (`party_bot::player_spawn`), laid out as retail's Duty Finder
  members' are. From Bob's Chronofoil capture of five Sastasha runs (2026-10-22), the other three members'
  SpawnPlayer, numbers only:

  | Field | Retail's members | A bot's |
  |---|---|---|
  | Object kind | Player (1, sub-kind Player) | the same |
  | Owner, tether | 0xE0000000 (none) | the same |
  | Content handler | the duty's (`INSTANCE_CONTENT`, 4 in Sastasha) | the leader's (the duty's in a duty, none outside) |
  | Display flags | 0x40000 with INVISIBLE (they spawned while the duty loaded), plus each one's own hide-head and visor | 0x40000 |
  | Battalion, ModelChara, BNpcBase, BNpcName, layout ID | 0 | the same |
  | Online status | each one's own (Party Leader, New Adventurer, Mentor) | Party Member (37) |
  | Current and home world | each one's own (40, 65, 57; one on 57 from 405) | the leader's home world |
  | Account and content ID | real ones | the party list's made-up ones (`0x00B0…` and the actor ID), so the list's entry and the character are one |
  | Title, voice, the two bytes after the GM rank (1–3, 9–11) | each one's own | 0 |
  | Level, class, HP, MP | synced (level 18), MP 10,000 | the bot's |

  Its actor ID is drawn from 0x10000000–0x1FFFFFFF, the range every player's has in Bob's captures (NPCs' are
  random), and not already used in the instance. A bot has no character in the database, so the client's requests
  that look one up, *View Search Info* (`RequestSearchInfo`, by content ID) and *View Adventurer Plate*
  (`RequestAdventurerPlate`, by actor ID), are now answered with nothing when there's no such character, rather than
  panicking the connection as the lookups (`unwrap`) would have.
* **Or an NPC with a player's look** (`kind=chocobo`, as before), spawned as retail spawns a **chocobo companion**: a battle NPC of sub-kind 3
  (`Chocobo`), owned by the player (`owner_id`), its name in the spawn's text and no BNpcName (`name_id` 0), character
  data flags 0x1, no display flags, battalion 0 (your side), ModelChara 0 (a human model built from its customize data
  and gear). Its nameplate then shows its name (Bob, in game, 2026-10-21). Retail's party-member NPCs (Trust, six in
  Bob's Chronofoil captures in territories 822 and 884) are sub-kind 9 (`NpcPartyMember`), owned, display flags
  0x40000, character data flags 0x2 and a BNpcName; spawned that way (`kind=partymember`) a bot's nameplate shows
  only "Lv100", since for that kind the client takes the name from the BNpcName. Player area attacks don't hit
  bots (they only hit `Enemy` NPCs).
* **A random look, of any race.** Each bot is one of the races, clans and sexes the character creator offers
  (CharaMakeType's rows, read from the game at runtime: every race, Hrothgar and Viera of both sexes too), with a
  face, hairstyle and ears or tail from that row's own lists (`GameData::creator_looks`): a menu's `Customize` is the
  look's byte it sets (5 face, 6 hairstyle, 22 ears or tail), its options the values (an option over 255, an icon's
  ID, counts 1 up to the number of options), and a hairstyle option is a CharaMakeCustomize row whose FeatureID is the
  byte. Other features and colours are random within the creator's smallest ranges. Without the creator's data, the
  first six races and faces and hairstyles 1–4, as before. (Workspace icarus features `CharaMakeType`,
  `CharaMakeCustomize`.)
* **Only gear its race and sex may wear.** Every item's EquipRestriction is a row of EquipRaceCategory, which says
  which races and which sexes may wear it (everyone, one sex, or one race's own gear). A bot's best gear leaves out
  what its race and sex may not wear (`GameData::wearable_restrictions`, `best_gear`), and so does its glamour: such a
  piece is left out and the slot keeps its gear's look. There's no model of another race's gear for it, so it showed
  as nothing (Bob's 2026-10-23 bundle: a Viera in a Miqo'te loincloth). The look is now chosen before the gear.
* **Hats a race can't show are hidden.** Viera and Hrothgar wear their own hat models, and can't wear the rest; the
  client then shows a stand-in (Bob's 2026-10-23 bundle: a Viera Pugilist's glamour hat, model 13). A Viera or
  Hrothgar bot whose head model has no file for its race and sex
  (`chara/equipment/e<model>/model/c<code>e<model>_met.mdl`, code 1501/1601 Hrothgar, 1701/1801 Viera) gets the
  hide-headgear display flag (`HIDE_HEAD`, as the headgear button sets), kept in its player spawn. Retail doesn't do
  this for players: in 3,532 players' spawns from Bob's Chronofoil captures, Viera and Hrothgar had their hat models
  sent as anyone's, the flag set only when they hid it themselves (about a third), so the client hides what it can't
  show itself.
* **A made-up name:** two words built from syllables (not the game's text), within retail's 20 characters.
* **Level 100** (`BOT_LEVEL`) with **the best gear its class can wear at that level**, found in the Item sheet at
  runtime (`GameData::best_gear`): for each slot (weapon or two-handed weapon, off-hand, head, body, hands, legs, feet,
  ears, neck, wrists, two rings) the item of the highest item level whose ClassJobCategory includes the class and
  whose LevelEquip is at most the bot's level, rarity then the newest breaking ties; PvP gear left out; no off-hand with
  a two-handed weapon; two of a ring that isn't unique, otherwise the best two.
* **Hit as a player is.** Enemies' hits on a bot are scaled as their hits on players, with a player's defences (its
  class's and statuses'); until 2026-10-23 they were unscaled, NPC against NPC, about eight times as hard (see
  [Combat](combat.md#enemy-damage-to-players)).
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
* **Out of combat** each bot has **its own spot in a sideways diamond around you**, as retail's Duty Support members
  kept (see [Party NPCs](party-npcs.md#out-of-combat-the-diamond)): the tank in front, the healer behind, the damage
  dealers either side, you in your own role's corner (`diamond_spots`: with you a healer, the tank 2 yalms ahead and
  the damage dealers 1 ahead and 1.72 aside), turned with the way you face; bots beyond the diamond's four corners stand
  in rows of three behind it. (Until 2026-10-23 they stood in a formation behind the party's tank, `FORMATION`: 35°
  either side of straight behind at 2.5 yalms, and so on.) It walks to its spot
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
  * **Getting out of area attacks** (`server/dodge.rs`, 2026-10-24, Bob: "they should move out of an AOEs way to a
    safe area as close as possible"): every bot, and every party NPC, standing in an area attack an enemy is casting
    (circles on the caster or where it aimed, cones, lines and rectangles: the shapes `in_area` resolves hits with,
    from the Action sheet's CastType, EffectRange, XAxisModifier and the omen's angle) sets off 0.8–1.3 s after it
    notices (at once if waiting wouldn't leave it time to get out at running speed), as retail's Duty Support members
    did ([Party NPCs](party-npcs.md#getting-out-of-area-attacks)). It runs to the nearest spot out of **every** area
    attack being cast, half a yalm past their hit edge (`DODGE_MARGIN`, beyond the half yalm `in_area` already
    counts): the shapes' own exits (straight out of a circle, off a cone's nearer side or out of its far end, off a
    line's nearer side, past its end or behind the caster) and rings of points around it every half yalm out to 40,
    keeping those on the navmesh; of the nearest (within a yalm of the very nearest), the one closest to its target
    (or the player). It walks itself there (`PartyBot::dodge`), sliding its own cast if it has one going; once out it
    holds that spot and still acts (NPCs' own acting runs for it, so casters and healers in reach keep going; melee
    ones wait), and 1.3 s after the last attack it was avoiding has gone off (`LINGER`, retail's) it goes back to
    fighting. Before this they stood in everything.
    **Only on the map** (Bob, 2026-10-24): a spot counts only if it's on the navmesh and a navmesh path reaches it
    (the best 8 are tried, `DODGE_PATHS_TRIED`), and the run follows that path, never a straight line off it; with no
    navmesh, no spot counts. **Nowhere to go**: it stays and braces once for that attack (`dodge::choose_brace`),
    shielding as many as it can: with two or more of the party caught, something for all of them (Reprisal, Feint or
    Addle on the caster; else Temperance, Succor or Medica around itself); with one, that or a shield on the most hurt
    one caught (Aquaveil, Divine Benison, Adloquium); else its own (Sheltron, Vengeance, Sentinel, Rampart). Only
    actions its class has learnt by its level (`GameData::brace_actions`: the Action row's ClassJobCategory and
    ClassJobLevel), each kept to its recast (`PartyBot::brace_ready`).
* **Its rotation** (`build_rotation`, a heuristic until bots' rotations are recorded), run by the NPC class rotations
  (`server/rotation.rs`, a 2.5 s pace like a player's GCD): its abilities whenever they're ready; its combos, each step
  after the one it follows (highest level first, so a combo runs to its end); its damage over time spells and
  weaponskills re-applied 2 s before they wear off (the duration in their script); and its filler: the highest level
  combo starter, or its strongest plain weaponskill or spell (the potency in its script), so Stone II replaces Stone.
  Single-target attacks only. Melee bots auto-attack (870), physical ranged ones too (872); casters don't.

## Glamours

Each bot can wear its own outfit over its gear: one picked at random, for its sex and class, from a list the server's
owner writes (`servers/world/src/server/glamour.rs`). The Chrome extension in `tools/glamour-collector` builds the list
from Eorzea Collection's glamour pages ([Tools](tools.md#glamour-collector-toolsglamour-collector)) and sends it to
Kawari Web, which writes the file ([Kawari Web → Bot glamours](kawari-web.md#bot-glamours-dashboard--bot-glamours)).
The list is **`glamours.txt` in the server's data folder**: `$KAWARI_DATA`, which in Kawari Web's container is
`/kawari/data`, the folder mounted from the host with `world.db` and `login.db` (without `KAWARI_DATA`, the working
directory). Before 2026-10-23's fix it was read from the working directory only, `/kawari` in the container, which
isn't mounted, so a file put beside `config.yaml` was never seen. It's kept out of git, as the item names in it are
the game's. The server reads it the first time a bot joins and again whenever it changes, and logs where it read it,
how many glamours, and any item, dye or class it couldn't find.

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
  class's weapon), and so are `Face:` lines (facewear isn't gear). Names match whatever the case or spacing, and either kind of apostrophe, with or without the
  `<...>` round them (the extension writes them); `undyed` means no dye.
* `Class:` (or `Job:`) takes classes and jobs by name, English name or abbreviation (`Black Mage`, `BLM`), `|`
  between them; several lines add up. It also takes the role groups Eorzea Collection uses: `Tanks`, `Healers`,
  `Melee DPS`, `Physical Ranged DPS` and `Caster DPS` (and `Disciples of War` / `Disciples of Magic`), worked out from
  the ClassJob sheet's Role and PrimaryStat (dexterity marks the physical ranged); `All Classes` means any class.
  Gatherer and crafter groups match no combat class, so those glamours are left out. A job and its base class count as one: a Black Mage glamour fits a Thaumaturge
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
* Bots don't follow you into another zone (they leave when you do). Bots from `!spawn playernpc` aren't level synced (only the Duty Finder's are); they're in the party list
  once invited.
* The party list didn't follow chocobo-look bots' HP (retail sends HP through the members' actors, which the client
  may or may not take from an NPC); shown as players, with the list's content IDs, it may: to be checked in game.
  PartyMemberPositions isn't sent for bots (no minimap dots).
* Bots don't know a dungeon's route: the tank bot follows you, so you lead.
* Bots always pass on coffers' loot (see [Coffers and Loot](loot.md#party-bots)) and don't open coffers.
* The tank doesn't Provoke (a role action, not in bots' kits), use mitigation, or pull packs before the player.
* No area attacks, buffs, role actions, mitigation, Raise, or dodging enemy area attacks.
* Targets are picked by these simple rules, not by watching retail Duty Support members; their rotations, heal
  thresholds and pull timing are guesses to be checked against a recording.
* Nothing has been tried in game yet: the look, the gear models and how the client shows an `NpcPartyMember` outside
  a duty are the first things to check.
