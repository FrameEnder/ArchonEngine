# Known issues and gaps

What is known to be missing, approximated or wrong in the fork. Remove an entry in the same commit that fixes it, and
add one when something is found.

## Bob's Chronofoil captures of 2026-11-12

* Not scripted yet: 802 (Broadening Horizons: only its last two talks are in any capture, 898dd0b9). 1003 needs none:
  retail finishes it with 3871 ([Quests](quests.md#quests-finished-together)).
* Quest battle 68 (Parley in the Sagolii, 2026-11-12) is from one won run: no lost run, the boss's 1098 and 1099 do
  nothing (no effect seen) and his 1056's statuses 293 and 294 last a guessed 30 s; not played in game.
* The Paladin's 1056-1059, the White Mage's 1080-1083 and 1162 (2026-11-12) are each from one run (1162 three), none
  played in game. Their enemies' timelines (`Lemur_6` … `Kamapuaa_1942`) are what they did in that run: the restless
  raptor and the Kamapuaa only briefly. The yarzon bleeders' 909 (cast on itself) showed no effect, so it does
  nothing; the green lavers' 316 gives status 151 for a guessed 30 s.
* Quest battle 63's sequence 3 waits for both the first wave and the boss under 70 %: in every run the wave fell first,
  so which one retail waits for isn't known.

## The Arcanist and Summoner quests (2026-11-12)

* Fourteen quests from Bob's Kawari Recorder export of 2026-11-12 (453-461, 1091-1095, 1103, 5466; `quests.md`) have
  no Chronofoil capture: their variables and notices are retail's, but most scene numbers come from the listeners'
  order and their flags are the fork's usual ones; the ones the export's field fights caught (455's 8 and 9, 458's 12,
  460's 22 and 23, 1094's 3, 5, 6 and 8) agreed. Grimoire Fandango's (460) scenes 14-36 are read from what each does,
  not that order; Over the Rails' (458) scene 20, a cutscene, and Shadowing the Summoner's (1093) 3, a
  ScenarioMessage, aren't played (nothing showed where); the bars of 458's and 459's landing boats are the sheet's
  EVENT_ACTION_WAITING2_MIDDLE, not seen. None is checked in game.
* **Their quest battles** (50-53, 79-81, and 65 for 521; `quest-battles.md`) come from one or two Recorder runs each:
  the rules that move them on (how many down, which health mark, how long after) are the ones that fit that run, and
  the enemies' timelines are what they did in it (an action seen once has no `every`). Not played: Sinking Doesmaga's
  (53) Doesmaga and his lackeys as event NPCs at the bow before his cutscene (the zone's own; retail took them away),
  Pincer Maneuver's (52) other grimoire (EObj 2002520) fading with the one picked up, Austerities of Earth's (80) dark
  summoner's Light Succor on his Ifrit-Egi (it heals only him), Y'mhitra's Stoneskin on you (79-81), the Echo's
  "strength beyond your limits" on a retry (81, 65; LogMessage 3212). Sequences retail never reached in these runs
  aren't scripted: 81's 4-6, 79's 6, Over the Rails' (51) talks 101 and 102. Statuses whose length wasn't seen: the
  Topaz Carbuncles' Curl (310) and Storm's stun, 80's Gravity Force's Heavy. Austerities of Wind's (81) quest line
  (LogMessage 3675) comes as its closing scene ends, as retail's; Escape from Castrum Centri's exit to Ul'dah faces
  rotation 0 (retail's facing isn't recorded). The ocn_o1e2 drop-in (51, 53: aboard the Morningstar) is new and assumes the
  level's LGB is `planmap.lgb`; none is checked in game.
* The hired fists and bow of 1093 (BNpcBase 1946, 1947) were never seen acting (Bob teleported away): their timelines
  are just the melee and archers' auto-attacks. The Allagan mirrorknights' Gust (917) never hit anyone, so its
  potency is a guess.

## Quest enemies from captures (2026-11-11)

* The 24 kinds of quest enemy added from Connor's captures (quests 986 to 3883, `quests.md`) have timelines from only
  a fight or two each: how often an action came is the median of the few gaps seen, or a guess at the pace where only
  one use was seen. 431 (BNpcBase 2893's 1.2 s cast) is left out, its effect not recorded; 878's damage type is a
  guess (slashing). None is checked in game.

## Upstream 7.58 (merged on `upstream-sync` on 2026-11-11)

* The server takes a 7.58 client (2026.11.03.0000.0000) only; nothing of 7.58 has been tried in game yet. Every fork
  opcode is found for it from Bob's executable ([Fork Opcodes](opcodes.md#finding-them-on-758)), none seen from a 7.58
  client or in a 7.58 retail recording. EventNotice2/4, EventResume16 and LotteryDailyBoard are members of groups the
  client handles identically, picked without a recording; a retail capture on 7.58 would say which retail uses.
* Upstream's EditSearchInfo (424, 7.57h1's) is corrected to 645 in `fork_opcodes.yml` (`corrects`); the build says to
  remove the correction once upstream changes theirs.
* The item operations are counted from ItemOperation's opcode + 1 (upstream's finding, 2026-10-31), 319 on 7.58: the
  fork's retainer, saddlebag, repair and gearset operations keep their offsets from it, unchecked on 7.58. The mail
  actions are the letter packets' opcodes (upstream's), unchecked on 7.58 too.
* Upstream's chat distances (Say and custom emotes 20 yalms, Yell 100, Shout everywhere, 2026-11-02) apply to players'
  messages; the world bots' and NPCs' own lines don't go through them.

## Upstream 7.57h1 (merged into `master` on 2026-10-21)

* Automatic opcodes after a game update ([Fork Opcodes](opcodes.md#every-opcode-after-a-game-update)) haven't run on a
  live update yet, only against Bob's 10.06, 10.16 and 11.03 executables. About 17 server packets come from candidate
  lists (the notices, EventResume2/4/8, InitResponse and PingSyncReply, the inventory packets): a wrong pick shows as
  that packet doing nothing (10.16 → 10.06 also leaves three Crossworld linkshell packets among 12 candidates). 14
  client packets aren't found and wait to be learnt by size. The scrambler tables aren't found automatically (no
  pattern found for them); the item operations and the mail actions follow the found opcodes since 2026-11-11.
* The fingerprints' second dispatchers don't read a `sub` then `cmp` on the opcode (7.58's ItemMarketBoard pair), so
  such a pair comes back as candidates.
* Four fork opcodes have no finder, because their code is the same as other packets': RequestPlayerRetainerInfo,
  GatheringPointPhase, ItemMarketBoardInfo and ItemMarketBoardSummary. After a patch only the fingerprints find them
  (the two ItemMarketBoards as a pair of candidates).
* Size-change warnings cover client packets only; a server packet whose layout changed in a patch isn't noticed.

* The merged server only takes a 7.57h1 client (2026.10.16.0000.0000). All the fork's opcodes are found for it (see
  [Fork Opcodes](opcodes.md#finding-them-on-757h1)), but only MarketBoardOpened and MarketBoardPurchaseHandler have
  been seen from a 7.57h1 client so far. EventNotice2 (834) and EventNotice4 (467) are retail's, from Bob's capture of
  the Limsa opening (2026-10-20).
  The fork's inventory operations (retainer entrust/withdraw/gil/market, `BASE_INVENTORY_ACTION` + 22..26) are
  assumed to keep their offsets from the new base (234), unchecked on 7.57h1.

## Duty Finder settings

See [Duty Finder Settings](duty-finder.md). Untested in game (2026-10-28).

* A party of real players still gets no bots for its missing roles (bots can't join a party the server keeps).
* QueueRoulette's layout isn't known: its settings are assumed to be at QueueDuties' place (byte 8) and only Limited
  Leveling and the loot rules are taken from it. A bundle with a roulette queued (Limited Leveling on) would settle it.
* The roulettes' daily bonus EXP (Leveling only) is a stand-in (`LEVELING_BONUS_EXP_FACTOR`), and the gil is the
  wiki's level 90 amount scaled down by QuestExpModifier, a guess; no tomestones or seals; no Adventurer in Need
  bonus; the bonus's own packets and log lines aren't recorded (no roulette in any capture).
* Lootmaster has no window: it plays out as Greed only.
* Minimum IL doesn't lower bots' gear; no capture used it.
* The Echo at a duty's start is known only for CFC 97, 101 and 107-110; the wiki's other Coils and older savage raids
  have none here. The Echo's wipe rule (three minutes, older trials) is the wiki's, not recorded.
* Echo's damage multiplier can push a hit past 65535 (the effect's amount is 16-bit): capped there.
* Explorer Mode: only the enemies, barriers and bots are handled; whether retail also stops actions, the duty's time
  limit (5400 s was still sent) or the coffers isn't checked.
* ContentFinderUpdate's and ContentFinderFound's retail flags and party makeup aren't sent (Kawari echoes the client's
  settings); the swift first-clear bonus (LogMessage 2163) isn't given.
* Bots waiting outside a duty with no room for them (a quest battle, a party bigger than the duty's) come back only at
  the next zone change after it.

## Party NPCs (Duty Support)

* Untested in game (2026-10-23). Whether the client builds its party list from ActorControlSelf 257, 276 and 1525
  alone, as retail's capture suggests, is the first thing to check.
* Recorded: the Marauder, Lancer and Thaumaturge (a healer's party) and the Conjurer (a tank's, 2026-10-28) of
  Sastasha, the Tam-Tara Deepcroft, the Copperbell Mines and the Bowl of Embers, at each duty's level
  (`party_npc::DUTY_LEVELS`); any other member chosen (other duties' members) is a Player NPC of that role, listed in a
  Duty Finder party list beside the party NPCs' slots. A duty not in `DUTY_LEVELS` gets them at their own level (15,
  the Conjurer 16). With a ranged damage dealer the client chose the Marauder, the Conjurer and the Thaumaturge in
  Sastasha (Bob's capture cc62ad94); a melee player's party isn't recorded.
* The Conjurer (2026-10-28) heals with Cure only: its Medica (29320; 1-9 times a duty on retail, with two of the party
  below 80 % mostly), Esuna and the instant Cure (29355) aren't cast. Its kit and heal line are medians from one run
  of each duty, untested in game; how hard enemies hit it isn't measured (`damage_taken` 1).
* With a player tank, Sastasha's tank lines (14298 at Chopper, 14303 after Denn) aren't recorded: nobody says them.
  Sastasha's start line comes 43.6 s in whoever says it; the Marauder's 14294 (said with no Lancer there) came at
  23.4 s in the one run that had it. The
  Bowl of Embers' Conjurer lines (14328, 14330) and the missing 13981 are from one run.
* The members' abilities that did no damage (26230, 29303, 29766, 29311, 29767, 25082, 25083) are only animations,
  and the Lancer's 29772 doesn't heal it. Their chatter in fights is at random every 150–400 s of fighting, not
  whatever made retail's members speak.
* The bots' diamond formation (also the Player NPCs') is measured with a healer player only.
* Getting out of area attacks (2026-10-24, `server/dodge.rs`) is untested in game, and so is bracing when there's nowhere to go (which actions retail's members used then isn't recorded; party NPCs brace with the player classes' actions by their class and level, not their kits). Every Action sheet shape is
  dodged as it's drawn (circles, cones, lines, donuts, crosses; [Combat: area shapes](combat.md)): a donut's hole read
  from its omen's name (`sircle_AABB`, AA across and a hole of BB) is the reading of the name, not checked against the
  client's marker yet, and a hitbox from the model is whichever formula the startup check finds (see its log line) and
  untested in game. Every area attack a hostile NPC casts
  counts, even one that would do nothing to them; the margin past the hit edge (half a yalm) and the reaction
  (0.8–1.3 s, quickened to 0.5–0.9 s on Bob's request) come from one fight (Tam-Tara's last boss, two casts). A bot's own cast isn't interrupted: it slides it. Dodging in turn and keeping to a boss room's floor (2026-11-04) are untested in game; only Brayflox's Longstop's
  last boss names its floor (`arena`), so in every other room a dodger with nowhere on the floor out of everything
  can still go off it (the other rooms' floors aren't known: most dungeons' layouts have no such range, and the walls
  alone don't say where the floor ends). A dodger doesn't weigh whether it can reach a spot before the attack goes
  off.
  The safe spot is checked only for standing on the navmesh, not for a path there that stays out of the area.
* How hard enemies hit them (2026-10-23) is the players' scale times retail's ratio for each member (a third for the
  Marauder), untested in game. On retail the enemies stayed on the tank (the Lancer and Thaumaturge took 330 damage
  each in 30 minutes); whether Kawari's enemies stray onto them more isn't checked.

## Fall damage

* Untested in game (2026-10-28). The formula fits the 19 retail falls whose height could be measured within 1.2 %
  (4 % at worst); falls higher than those measured aren't checked. The drop is from the last
  height the player stood at, as Kawari reads the client's UpdatePositionHandler: a fall that starts from a slope or
  a jump off a ledge counts from where they last touched the ground. Two retail landings 10.88 and 14.43 yalms down
  did no damage (perhaps onto a slope the client slid down, or into water); Kawari would hurt both. Falling into
  water, gliding and flying aren't checked: only mounted, dead and gimmick jumps are spared. In combat is Condition
  InCombat as Kawari sets it.

## Aethernet destinations

* Untested in game (2026-10-28). The gates and airship landings are taught after the last shard is attuned, after a
  quest completes and at login; a character that already had every shard is taught them at its next login. Whether
  the aethernet menu lists them at once without a re-login isn't checked. Rows with `Invisible` set outside the three
  cities' groups (later cities) are taken the same way, unverified.

## Facing at a duty's entrance (2026-10-30)

* Retail's facing is a recorded table ([Dungeons](dungeons.md#facing-at-the-entrance)); a duty not in it still faces
  the way its entrance rect does, which was wrong in most duties checked. What retail's facing comes from isn't known.
* Six weren't recorded (the player had already moved before the first position update). Haukke Manor's (1040,
  2026-11-02) is one run's ActorSetPos. Untested in game.

## Client packets of the wrong size (2026-10-30)

* A client packet whose layout doesn't match the client's size now only warns ([Packet layouts](packet-layouts.md#client-packets-of-the-wrong-size-2026-10-30));
  the fields of a short layout read zeros past the client's bytes. CrossRealmListingsRequest2 (878) is 64 bytes on the
  client and 40 in upstream's layout, so it warns each time the Party Finder asks. Which packet the Waking Sands' door
  click came with in Bob's bundle isn't known (nothing logged the bytes); nothing else in the captures differs.
* Several packets in one TCP read ([Packet layouts](packet-layouts.md#several-packets-in-one-read-2026-11-10)): the
  lobby's fix (`take_packets`) and the zone and chat connections' (2026-11-10, the lost connection after a duty) are
  untested in game. The first reads of a zone connection (its setup, before `client_loop`) still take one packet each.
* A coffer's pieces come as its bar ends; retail's came 1.1 s after it (the opening animation). Items' own cast times
  (CastTimeSeconds) are untested in game: if the coffers' is 0 in the sheet, they still open at once.
* Weapon coffers: only the "Level N Weapon Coffer"s are filled in, from the wiki; the raid and tome coffers say their
  contents aren't known. Opening one on retail was recorded once (the Level 18, a gladiator's).

## Chocobo porters (2026-10-30)

* Untested in game. Built from nine retail rides (three routes); see [Travel](travel.md#chocobo-porters).
* Only stands 5, 15 and 19 have their route start; elsewhere the ride starts where the player stands, which may be
  off the route the client rides. A capture of each stand's ride gives its start.
* Which stands register another along with them (5 and 3 with 2, 24 with 21, 15 with 10) is a table of what was
  seen; the rule behind it, and the stands not seen, aren't known.
* The mount, the online status and `ChocoboTaxiStart` go as the client finishes zoning, not with the ActorSetPos
  before it as on retail. Whether the client minds isn't known.
* ActorControlSelf 263, 0, 928, 504, 930, 253 and 260 that came with retail's ride aren't sent, nor the
  EventFinish's 0x12.
* Logging out or disconnecting during a ride isn't handled: the character is saved wherever it was on the route.
* The checkpoints are only logged; nothing checks the client rode the route.

## Flying and riding speed (2026-10-31)

* Kawari sets ZoneInit's flying flag only where the character has completed the zone's AetherCurrentCompFlgSet, as
  retail does. Nothing but the GM commands grants set 19 (A Realm Reborn's) yet: how retail completes it (presumably
  with A Realm Reborn's story) isn't recorded. Riding speed should follow (it was faster with the flag wrongly set);
  not yet checked in game.

## Party bots

* Stage 1 only (see [Party Bots](party-bots.md)): `!spawn playernpc` bots are in the party list only once invited,
  aren't level synced outside duties, don't follow you into another zone unless they're in your party, use
  single-target attacks and one heal only. The Duty
  Finder doesn't fill roles with them yet. In combat melee bots still stand on the same spot by their target. Bots are
  spawned as retail's chocobo companions are (for their nameplates' names): whether the client treats them as your
  chocobo anywhere else (the companion's bar, its commands) hasn't been checked. They run at NPCs' speed (6 yalms a
  second), so they fall behind when you sprint or ride.
* Bots' rotation scripts (2026-11-10) cover the ten jobs of A Realm Reborn and their classes; untested in game, checked
  with a lupa simulation against a dummy and against retail players' uses a minute. Not yet: the Heavensward and later
  jobs (Dark Knight, Astrologian, Machinist, Samurai, Red Mage, Gunbreaker, Dancer, Reaper, Sage, Viper, Pictomancer:
  no bots of them yet either), and, for the ten, what needs a gauge or system the server hasn't got: the Ninja's
  Ninjutsu is one spell (no Suiton before Trick Attack, no Huton, Doton, Katon by count), the Summoner's arcana and
  attunement are counted in the script, not the server (Gemshine is always the Ruby one), the Monk's Furies are counted
  in the script, the Bard's songs don't reset Bloodletter or build Repertoire and Soul Voice, the Paladin's Oath isn't
  kept, nothing past 50 on any gauge (Enochian, Polyglot, Paradox, Astral Soul, Beast Chakra, the dragon's eyes,
  Ninki, Kazematoi). Role actions (Lucid Dreaming, Second Wind, Swiftcast...) aren't used. Ability charges (Infuriate,
  Mudras, Bloodletter) are one at a time.
* Party bots' heals are still the bots' own choice (`heal_action`), not part of the healers' scripts.

## Quests

* **The 50 quests of Connor's Chronofoil captures** (2026-11-09: 521, 522, 978-986, 1001-1005, 1037, 1175, 1189-1192,
  1346-1363, 1442-1448, 3871-3884, 4521, 4522; Bob's dumps of 2026-11-09): untested in game, and each from one run
  (978-983 also from Bob's capture 689b15da). Written by `questkit transcribe`, so a line retail didn't play isn't
  there: talking to someone in a sequence where retail didn't does nothing, and the client's gating (IsAcceptEvent)
  wasn't checked step by step. Things done in any order are only where retail's variables showed a count with a flag
  each (talks, emotes, key items, kills); 1001 and 3871 were written by hand from their listings. Moves with no pop
  range of the quest's go to retail's spot (`change_territory` with a position). Quest battles 62 (983, three
  runs), 63 (1004, two) and 64 (1005, one: no lost run, and its worn-down ally never acted) are scripted, their
  territories taken to be copies of Coerthas and Mor Dhona from where their spots stand; 64's two talking event NPCs
  aren't put in the battle. 1651's 129, 1655's 527, 1874's 91 and 1988's 1286 give statuses whose length wasn't seen.
  **Not scripted**: quest battles 5060 (Cape Westwind, 4521) and 5061 (Devastation, 4522), so those quests stop at
  their battle (65 is since 2026-11-12, `quest-battles.md`); and the duties Castrum
  Meridianum (InstanceContent 15, 3873), the Praetorium (86) and the Porta Decumana (20082, 4522), Thornmarch (Hard)
  (20011, 1190) and the Whorleater (Hard) (20017, 1361): their quests move on when one is cleared and let out where
  retail did (`set_duty_exit`), but the duties themselves don't exist yet.
* **The 21 main scenario quests of 2026-11-06** (518, 876, 878, 883, 884, 886, 887, 889, 890, 897, 910, 911, 912, 924,
  927, 938, 939, 940, 941, 3868, 3869; Bob's dumps and his two characters' runs, Chronofoil 130088ec and 3c56e09f):
  untested in game; a third character's run (Chronofoil 21f967d2, 2026-11-07, 876 before 518) matched every step,
  bar and scene. Lines for NPCs retail didn't play are numbered by the client's IsAcceptEvent order (Bringing out the
  Dead's ACTOR14-16 and 20-23 weren't talked to in any of the three runs).
  876, 890, 924, 3868 and 3869 complete 877, 896, 925, 926, 917 and 937 with them (retail's FinishQuests, no rewards).
  questkit's fixture for Following the Evidence (911) had EOBJECT3 and 4 the wrong way round: `fixture` matches a
  step's places to its listeners in the Quest sheet's order, which isn't always right (fixed by hand from the
  variables and scenes; since 2026-11-07 an existing fixture's base IDs win, so later runs keep the fix). Factual Folklore's key item on the beast (`event_item_effects.json`) gives retail's statuses
  and the fifth of its health, but what statuses 22 and 62 do to it isn't modelled.
* **The Waking Sands' 92219** (2026-11-06): it came in ZoneInit with Bringing out the Dead at sequence 2, the only visit
  after All Good Things in the captures (the third character's, 21f967d2, didn't go there); the layer that carries it belongs to ManFst405, a quest neither capture had,
  so it's taken from 876's acceptance (`layer_filters.json`). ManFst405 is Reclamation (520), whose completion moves
  the key on to 92220 (2026-11-11, `travel.md`; untested in game); what brings 92219 in before it is still the guess.
* **857, 517 and 3867** (2026-11-06, from Bob's dumps and three runs each, Chronofoil 650dbfd8, 0fd8bce0 and 48aaf6fe):
  untested in game. Lord of Crags' sequence 255 comes as the player leaves the cleared Navel (`onDutyCleared`), where
  retail sent it as Titan fell, as for the other duties' quests. All Good Things and You Can't Take It with You
  complete 871, 872 and 881 with them (retail's FinishQuests, no rewards); whether the client lists those as done
  depends on the completed-quest mask alone. Lines whose speakers retail didn't play are numbered by the client's
  IsAcceptEvent order.
* **850, 855 and 856** (2026-11-05, from Bob's dumps and three runs each, Chronofoil d65126c9, b4ce5e3f and 811eb9ff):
  untested in game. In the Company of Heroes has a line for everyone in every sequence, but retail was only seen
  playing the quest's own scenes: the rest are numbered as the client's IsAcceptEvent orders its targets (which fits
  every scene the client has), and the feasting tables' scenes are empty, so a wrong number there shows as nothing
  happening. As You Wish's destinations are put out by the client itself (no SpawnObject), as Battle Scars' palm wines.
  It Was a Very Good Year's enemy (base 1910) has a timeline from three fights; its hitbox isn't recorded.
* **The six quests after Brayflox's Longstop in Eastern La Noscea (3866, 840, 843, 845, 846, 848)** (2026-11-05, from
  Bob's dumps and three runs each, Chronofoil b766138c, 811b6f1b and 62c1dfe4): untested in game. Battle Scars' palm
  wines (layouts 4293291-4293293) are put out by the client itself on retail (no SpawnObject in any run); the server
  maps them to EOBJECT0-2 from the zone's layout, which replay can't check. That Weight counts any tainted water
  sprite the player kills; whether retail counts one killed by a party member wasn't seen (Bob was alone).
* **Ul'dah's Most Wanted (261) and its quest battle 19** (2026-11-05, from Bob's dumps and Chronofoil capture
  a763f0a0): untested in game, from one run. The second wave after the second of three thugs is down, and the 2 s
  the director waits on a death, fit that run; the quest's thugs' Feint gave Slow (status 9) on retail, which Kawari's
  Feint doesn't (nor is Slow modelled); how often the thugs' Feint and Heartstopper and the tougher thug's True Thrust
  come is a guess (each was used once). The gutless gladiator can be talked to again while his two are out (they
  aren't brought out twice); what retail does then wasn't seen.
* **On Holy Ground (264), The Rematch (265), their quest battles 22 and 23, and Paladin's Pledge (1055)** (2026-11-06,
  from Bob's dumps and Chronofoil capture accc33a9): untested in game, from one run each. 23's sequences 3 and 5 come
  where retail's player was (near the third pair, near the archers below the camp), which fits that run only; whether
  the ones along the way take the player on as they come close, and the background soldiers keep at each other, as on
  retail, is up to Kawari's aggro and the director's targeting, unseen. In 265 the second place searched first marks
  itself in UI8BH (a guess). The new NPC actions' statuses (59, 60, 99, 476, 829, 29167) last a guessed 15-600 s and do
  nothing. 1055's Paladin soul crystal comes from `finish_quest` as Seer Folly's does; Paladin's Pledge's
  GEARSET_UNLOCK (1905) isn't sent (what it unlocks wasn't looked at).
* **That Old Familiar Feeling (262), The Face of Thal (263) and their quest battles 20 and 21** (2026-11-06, from
  Bob's dumps and Chronofoil capture 941895d2): untested in game, from one run each. What brings each wave fits that
  run only: 20's second wave on a 30 s timer after sequence 3 (its first five weren't all down), both battles' third
  waves at once on the second wave's fifth (20) or sixth (21) down, 21's win on its leader's health reaching 0; the
  director hears of the first waves' deaths 2 s late, as 19's. Which way 254 and 255 are copies (Steps of Nald,
  Western Thanalan) is by the run's positions on their ground, not their TerritoryType. The NPCs' rarer actions
  (once or twice in the run) come about as often as that one run had them. Flash's blind, Fluid Aura's bind and the
  NPC Shield Bash's stun last a guessed 12, 6 and 6 s, and what statuses 13 and 15 do isn't scripted. In 263 the
  point of ambush found before both talks waits for them (UI8BH and BitFlag8 3, the client's todo); retail talked
  first.
* **Seer Folly (1079) and its quest battle 75** (2026-11-05, from Bob's dumps and Chronofoil captures 8d550e33 and
  dc565f7c): untested in game, from one run. What brings the second wave (the boss's adds down) and the third (the
  boss at 40 %) fits that run only; the QuestBattle sheet's other ten layout IDs (more raptors, bats, slugs and water
  sprites) never came, so whatever brings them (a bigger party? a longer fight?) isn't known and they aren't spawned.
  The allies' Regen, Stoneskin and Divine Seal aren't played, and the slug's Silence isn't enforced by the server
  (the client may grey out spells itself). Which of the empty scenes 3-5 EOBJECT0 and A-Ruhn-Senna play is a guess.
  ActorControlSelf 540 (sent with the soul crystal, the quest's HOW_TO_JOB) isn't understood. Equipping the soul
  crystal sends the class change as a weapon's does, with Kawari's ack first and no map markers (retail's ack comes
  after ActorControlSelf 57, and its map markers twice). Whether retail puts a job's crystal on by itself when a
  plain class weapon is equipped (a cane taken out while a conjurer, the White Mage's crystal in the armoury) wasn't
  seen; Kawari doesn't (only for a job's own weapon, as before). The class from the soul crystal over a gearset
  (2026-11-10) is untested in game.
* **The Lominsan Way (59)'s bomb phase** (2026-11-04, Bob's report after paralysis was fixed): Rive aimed at the
  player, 1769 hiding from the bomb and coming back, and the bomb's countdown lines are retail's now, untested in game.
  The Rive retail used from the hiding place about 3 s after the bomb was gone isn't played; what 1769 does if the bomb
  blows and 1771 lives wasn't seen (it comes back as after a kill). battle-sim doesn't check log lines or movement.
  Rive's knockback (2026-11-04) is untested in game, as is Rive keeping its facing when Heavy Swing comes the tick
  it goes off; the entry retail sent after it (kind 27, `00 80`, 1299) isn't.
* **Joining a class from another (177, 285, 286; the CLASSJOB unlock)** (2026-11-04, from Bob's dumps and Chronofoil
  capture d3379099, a Conjurer): untested in game. Only the Gladiators' route is scripted; the other classes' joining
  quests unlock their class the same way once they are. The server doesn't stop a locked class's weapon being
  equipped (the client refuses it). What 177 does for someone the client says
  doesn't qualify (its IsQuestAcceptQualified) wasn't seen: the answer it returns decides.
* **Changing class** (2026-11-04, Bob's bundle 20261104-135324 and Chronofoil capture d3379099): the reply is in
  retail's order now ([Quests](quests.md), "Changing class"), untested in game: whether that is what let the client
  change class again isn't confirmed. Still unlike retail: the InventoryActionAck for a weapon comes first (retail's
  after Equip and ActorControlSelf 57), the Equip and the stats come once more than retail's, and what
  ActorControlSelf 805 (after a gearset) and 3103 or 3101 (after its equipped container, by whether it has a glamour
  plate), all sent, mean isn't known. A gearset's moves are retail's now ([Quests](quests.md), "A gearset's moves"), untested in game; a slot the
  gearset leaves empty is sent as a Move from Equipped to the armoury, which no retail gearset showed, and the
  transactions' sequence is Kawari's item sequence (retail's is a counter of its own, e.g. 0x84482). The
  level-31 "All Classes" armour stayed on the level-1 Gladiator (`remove_incompatible_armor` checks the class, not the
  level); whether the client takes it off itself wasn't seen. A summoned pet isn't dismissed on a class change (only
  its hotbar is cleared, as retail's packets do). Another player's Equip carries the receiver's class.
* **The Gladiators' first two quests (253, 256)** (2026-11-03, from Bob's dumps and Chronofoil capture 208c3716):
  untested in game; each replays against the one run, played by a level 37 Gladiator, so 256's enemies' damage wasn't
  recorded (their timelines use the defaults). 256's marauders and Pfarmurl take any emote (the Quest sheet's emote
  listeners have no condition; retail's was 23). Its second sack's bit in UI8F (0x40) and the companions' empty
  scenes (13-16) are taken from the client's order, not seen.
* **Wrath of the Titan (516) to The Things We Do for Cheese (832)** (2026-11-03, from Bob's dumps and Chronofoil
  captures 44df02fc and c9145fdc): untested in game; each replays against both runs. The quest enemies' timelines
  (`GaiUsb*Enemy_*`) are from two fights each (one for 1889's 426, at retail's param 100 where its shared script gives
  50; one for 1887's 347). Trial by Turtle's EOBJECT1 and Hungry Hungry Goobbues' EOBJECT1-5 weren't used on retail and
  do nothing here. Wrath of the Titan's warp after its accepting cutscene is the Waking Sands' layer filter moving on
  (90344, one capture): anyone already past 516 gets 90344 there too.

* **The Twelveswood's story, Spirited Away (725) to Into the Beast's Maw (514)** (2026-10-30, from Bob's dumps and
  Chronofoil capture caa5e908): untested in game; each replays against its retail run and two more of other
  characters' (captures 1fac2416 and 2528a32d, 2026-10-30; `<quest>-1fac2416.json`, `<quest>-2528a32d.json`). Into the Beast's Maw needs
  the Thousand Maws of Toto-Rak, scripted since 2026-10-30 (see below) and untested.
  Sylphish Concerns' second destination's nibble (UI8CL) is a guess (it was the last in all three runs; the client's
  script never reads it). The quests' humanoid enemies (BNpcBase 55, 61, 62,
  64) use the Western Thanalan 4th Cohort's timelines (the same bases), not numbers from these fights.
* **The Twelveswood's story, We Come in Peace (3856) to Brotherly Love (724)** (2026-10-30, from Bob's dumps and
  Chronofoil capture 5fcc970c): untested in game; each also replays against capture 2528a32d's run
  (`<quest>-2528a32d.json`), and each but We Come in Peace against 1fac2416's (`<quest>-1fac2416.json`). The fixture
  maker read Presence of the Enemy's (719) kills in 2528a32d out of order and missed the last (it came 0.08 s into the
  box's scene, before it was answered): the fixture was put right by hand from the kills' ActorControl 6. Retail completed the old
  numbers of three (708, 717, 718) with the new ones, which Kawari does too (`mark_quest_completed`); what the old ones
  did on their own isn't known, and they aren't scripted. First Contact's (715) other emotes, Dance Dance Diplomacy's
  (3857) other-emote scenes and Presence of the Enemy's (719) box before the four are down weren't seen on retail.
  Retail's quest enemies' actions 364 (the ochu), 404, 475, 308 and 403 (719's ambushers) aren't scripted, so they're
  left out of their timelines; those timelines are from one fight each.
* **An event that combat breaks off** (Sylphish Concerns' first search in capture 1fac2416: an enemy hit the player 0.14 s
  into EOBJECT0's first scene): retail ended it with EventFinish result 11 and ActorControlSelf 140 `[handler, scene, 1]`,
  and the player searched again. Kawari doesn't break events off for combat, so neither is sent; the fixture leaves
  that attempt out.
* **The Company You Keep (Immortal Flames)** (682), **For Coin and Country** (685) and **Sylph-management** (513),
  2026-10-28: played through in game by Bob (bundle 20261028-220355, a conjurer); the others' lines in 682 go by
  IsAcceptEvent's order, not recorded.
* **A Hero in the Making** (511, 2026-10-28): played through in game by Bob. Retail talked to Tataru in territory 140
  (where she stands for sequence 1); the officers' lines (scenes 3-5, 30-32, 35-37) go by IsAcceptEvent's order, not
  recorded.
* **My Little Chocobo under the Scenario Guide** (2026-10-29): the quest stayed listed under the guide once done.
  Retail completes a quest before ending the event; Kawari's scripts ended it first. The event now ends after the
  quests finished with it, which is believed (not yet seen in game) to be what the client refreshes the list on.
* **The Gold Saucer** (2026-10-29, [The Gold Saucer](gold-saucer.md#known-gaps)): untested in game. No chocobo racing,
  no Triple Triad (437 isn't scripted); the Jumbo Cactpot's claim is a guess (paid at the next talk) and its participant
  bonus missing; the Gold Saucer window shows nothing; no Gold Saucer bonuses. The Mini Cactpot's diagonal lines (6, 7)
  and The Finer Miner's Morbol and Titan weren't played on retail.
* **GATEs** (2026-10-29, [The Gold Saucer](gold-saucer.md#gates)): untested in game. Only Air Force One is played, so
  every slot holds one; Cliffhanger, Any Way the Wind Blows, Leap of Faith and The Slice Is Right aren't. The Gold
  Saucer's Air Force One (rows 74-76, territory 832, at :20) wasn't seen on retail since 7.4, only the Cieldalaes'. The
  entrance's first parameter and the opening's fifth are retail's numbers, not understood; the rank in the result is a
  guess (1 for S, as both runs); RideShootingEnd is retail's 40 bytes as they were. Where the duty puts the player isn't
  known (InstanceContent 62005 has no entrance; Kawari picks a pop range.) A GATE played is remembered for the
  connection only (logging out and back in lets it be played again), and from the Duty Finder's question rather than
  entering. The
  entrance's spawn lacks retail's 0x40000 display flag at the GATE's start; the place the GATE Keepers put you is 4.5
  yalms straight in front of the entrance (retail's was a little to one side.)
* **Fixed (2026-10-29, Bob's bundles 20261030-010154, -010830, -010939)**: the GATE Keepers' warp ended at the
  aetheryte, the entrance stood out all the time and couldn't be talked to, the screens didn't show the GATE; every
  arcade machine stopped at its coin (GoldSaucerArcadeMachine was read without its language, so no row was found); the
  Mini and Jumbo Cactpot brokers stuck in their quests' talk (Scratch It Rich and Hitting the Cactpot called
  `has_quest` with the event ID, which the server refuses; questkit's harness now refuses it too.)
* **The Mini Cactpot** (2026-10-30, Bob's bundle 20261030-102912): the first talk ever is retail's explanation, which
  ends the talk without a ticket (as on retail); the second sells one. The ticket itself never showed, the talk stuck
  until `!finishevent`: LotteryDailyBoard's 19 and 23 were two bytes early (fixed, not yet tested in game). Whether
  retail's explanation sells a ticket to someone who already has MGP isn't known (retail's player had none).
* **The MGP Voucher** (2026-10-29, Bob's bundle 20261030-021002): did nothing; fixed, not tested in game, and its
  packets on retail weren't recorded.
* **Warps** (2026-10-29, [Travel](travel.md#warps-and-their-conditions)): the ConditionFailEvent's packets are a guess
  (retail's success talk's); a warp with a ConfirmEvent doesn't play it (retail's airship and ferry didn't either.)
* **Mounting** (2026-10-29, [Chocobo companion](chocobo.md#riding)): retail's ActorControl 0 `[0, 1]` and 2 `[4]`
  around the Mount packet aren't sent, nor what it re-sends when you ride into a zone that has mounts. A companion put
  away by riding one's own chocobo into a zone without mounts isn't brought back. Places within a zone where mounts
  aren't allowed aren't known to the server.
* **The chocobo's colour** (2026-10-29): sent at login (ActorControlSelf 274); recolouring it isn't done.
* **The Scenario Guide** (2026-10-28, [Quests](quests.md#scenario-guide)): retail's order of the Grand Companies' three
  after 511 (681, 680, 682) is sent since 2026-10-29; whether the guide then shows 680 or 682 once accepted (rather than
  "Select a Grand Company") isn't known, on retail or here. The quest completed last is worked out from the completed
  quests, not recorded as retail must; a quest done out of the story's order (with `!quest` or ArchonEngine
  ControlPanel) could put the guide on an earlier step. Steps before the city's last quests and after We Come in Peace
  weren't in a capture. We Come in Peace (3856) isn't scripted.

* **The Waking Sands' layer filter** (2026-10-26) is known from The Scions of the Seventh Dawn (90338) to
  Sylph-management (90342, 2026-10-28); later visits get 90342 until their keys are recorded (a capture of retail
  there, or the zone's filter log matched to the story). 90341 is keyed to Wood's Will Be Done (683) and the other
  companies' 684 (untested) and 685 (Bob's capture c99fe374, 2026-10-28); whether accepting 680 or 682 already brings
  it isn't recorded. Other zones with several layer filters send 0, as before.
  The server's own event objects there still follow the first filter for the territory, not the key. Untested in
  game.

* **Into a Copper Hell** (660) relies on quest battle 44's director (another branch) to set sequence 5 when the
  battle is won; without it the quest stays at 4 after the battle. Its duty step, like 245's, is forgotten if the
  player logs out inside the cleared Copperbell Mines. EOBJECT0's empty scene 11 (seq 4) does nothing here.
* **Absolutely Glamourous** (3018): retail only showed the offer (declined). The rest follows the client's script and
  3017: where ITEM0 sits at sequence 2 (UI8BL), the notices (sent without the item icons, which aren't known), and
  that Swyrgeim's scene 5 hands over ITEM1, are unverified.
* **A Wild Rose by Any Other Name** (510) moves the player after cutscene 50 with `change_territory` to the recorded
  spot (warp type Normal; retail's ActorSetPos used warp type 18). 510's ACTOR1 talk (scene 1) and 509's Tataru line
  (scene 5) were not played on retail.
* NPC trades of inventory items: the item stays in its bag until the script calls `take_handed_in_items()` (only Color
  Your World, 699, so far; the other scripted trades hand over key items). A new quest that trades an inventory item
  and forgets the call gives its reward for nothing.
* **It's Probably Pirates** (245) passes replay but hasn't been played in Kawari yet. Its duty step moves on when the
  player leaves Sastasha after its clear (`onDutyCleared`), by the exit or any other way out; retail's run only showed
  the exit. Logging out inside the cleared duty forgets the clear, so the quest stays at sequence 4. The
  sailors' and Yellowjacket's lines meanwhile (scenes 4, 6–20) are matched to them by order, unseen on retail. The
  quest's other unlock parameters (UNLOCK_LFG_PARTY, COLLECT_PARTY_UNLOCK, UNLOCK_ADD_NEW_CONTENT_TO_CF, the screen
  images) sent nothing on retail: the client shows them itself.
* Gridania's Lancer and Archer **Close to Home** (85, 123) and Limsa Lominsa's Marauder one (108) come from their client
  scripts only, with fixtures adapted from 124's and 109's (no retail recording). Gridania's three still attune the
  simple way (no bar or nested aetheryte event, as 124's 2026-10-14 fixture showed).
* **Making a Name** (111) is the only way out of the Limsa opening's zone (181) in Kawari, as in Bob's retail run; if
  some other exit exists on retail, it isn't recorded. Its ferry back (scene 15, sequence 2) is from the client script
  only. Retail sent no AcceptQuest for it, only the journal update; Kawari's `accept_quest` always sends one.
* Retail tells a new Limsa Lominsa character who to talk to next (an unnamed server packet, opcode 533 on 7.57h1:
  a count, then each offer's ID, the NPC's layout ID and the quest, e.g. Close to Home and quest 111 after Coming to
  Limsa Lominsa). Kawari doesn't send it. It's the two-marker size of upstream's UpdateMapMarkers4/8/16/32 (32 bytes:
  the count, then two map icons, two layout IDs, two quest row IDs, 4 bytes of padding; 2,600 of them in Bob's 10.16
  captures, in every zone, at zone-ins and after quest steps). Its icons are 71021 (732), 71001 (179), 71141 (135),
  71151 (53) and a few others, and none of its quests was one the player had accepted (only 63 were accepted later, 61
  of them 71001s): they look like offers to pick up. Which quests a zone shows needs the Quest sheet (the issuer and its
  Level, the icon's EventIconType), which isn't here to check against; its 7.58 opcode isn't found either.
* **The Gridanian Envoy** (507) and **Call of the Sea** (674) were written from Sapphire and the recording without the
  client's script: dump them in the Quest Lab to check the scene numbers. Envoy's arrival positions are Sapphire's.
* Story quests stop after **Call of the Sea** (674), which hands over to Limsa Lominsa.
* Limsa Lominsa's story now runs to The Lominsan Envoy (546), the end of its own part, with every side quest in Bob's
  capture of 2026-10-21 (Rocks in a Hard Place, 471, from his dump of 2026-10-22). Retail also marked quests 398 and
  404 completed along with 397 and 402, as 464 with 543 (the scripts do it with `mark_quest_completed`); why (the
  sheet, presumably) isn't known.
* 412–415, 471, 544–546, 689 and 690 and quest battles 24 and 36 (2026-10-21) are untested in game; they pass replay and
  battle-sim. Their NPCs' and objects' base IDs were matched to retail's layout IDs by the scenes they play. Guesses:
  quest battle 24's territory 249 is taken for a copy of Lower La Noscea and 36's 280 for Western La Noscea (their NPCs
  are in those drop-ins); 24's first two corsairs fight once hurt (retail: when they noticed the player); 36's second
  half starts with `warp_players` where retail used a loading screen to the same spot. Bob's runs of 24
  (2026-10-22): the crash, the healing from afar and the fight through the cutscene are fixed (his third run); the
  distances that start each part, the conjurers' damage over time and the thaumaturges' Fire are fixed since, not yet
  retried in game. Sequence 3 came 4 s after the first arrow in the lost run and 1.3 s before it in the won one, so
  it isn't tied to their noticing you alone. Your own Ruin hit the corsairs for 46–50 where retail's was 38–43: Bob's
  gear (retail: Intelligence and attack magic potency 53, HP 223, defense 46, magic defense 82; Kawari 58, 245, 60,
  99; PlayerStats in the capture and the bundle, both Arcanist 14), about what 9 % more Intelligence gives. The boss,
  though, took 47 on both: retail's Ruin hit it harder than the corsairs with the same gear (47 to 41), which Kawari
  doesn't do (a lower defense for it, perhaps; not known). Damage *taken* doesn't depend on the player's gear in
  quest battles (recorded amounts). The boss and some adds have actions without scripts (see
  the battle's page). Adds still standing when a quest battle is won are stopped (`end_fight`); what retail does with
  them isn't recorded (every capture's won battles had none left), so they stay where they are rather than vanish.
* Quest battles' opening scenes now use retail's flags and EventStart flags (the UI stayed up, no skip button);
  untested in game, and which of the two the skip button needs isn't known. Quest battles 18, 37, 38 and 39 have no
  recorded opening and use the most common flags. Without a Doubt takes emote 12
  as the right one (the only one seen). The Lominsan Envoy's Zanthael yes/no (scene 2) is taken as the way back into
  the Command room, and its ticketers' later flights land at the next city's arrival, both from the client script
  only; A Mizzenmast Repast's scene 5 and Feint and Strike's EOBJECT0 (empty scenes) aren't played. Righting the
  Shipwright's ambushers use a timeline made from that fight (`RightingTheShipwrightAmbusher_170`), though base 170
  is in 19 open-world spawns too.
* 397, 402, 403 and 406 (2026-10-21) are untested in game. Their ambushers (402's two, 403's one) use the existing
  timelines of their BNpcBase (`TreeSlug_39`, `Bomb_17`), not ones made from these fights. 402's and 403's objects are
  also recognised by retail's layout IDs, in case their base IDs were matched wrongly (as Double Dealing's sack was).
* 466–469 (2026-10-21): their NPCs' and objects' base IDs were matched to retail's layout IDs by the scenes they play
  and the Quest sheet's listeners (the capture doesn't say). Double Dealing's sack of oranges was taken for EOBJECT2
  at first, so it did nothing (Bob's 2026-10-21 bundle); Loam Maintenance's three objects are matched by order
  (layouts 3929266–68 to EOBJECT0–2), untested. Double Dealing's emote takes any
  emote (the listener names none; Bob used 12); its EOBJECT1 and the EVENTRANGE0 exit listener weren't used
  on retail and do nothing; its ambushers lack the quest fields retail's SpawnNpc gives them (the player as owner,
  EVENTRANGE0 in the event field, flags 2), like the other quest-spawned enemies below. Plowshares to Swords counts every kill of ENEMY0 or ENEMY1 (each gave an
  item on retail, three of three; the listeners are drops, so retail may have a chance). Just Deserts' one-liners for
  the other NPCs (client script only) are played with HIDE_HOTBAR, a guess.
* **Just Deserts' quest battle** (29): territory 272 is taken to have Middle La Noscea's Bg, as 252 (so it loads
  `sea_s1f1`'s `QB_SubSea057_001`); if the golem doesn't appear, that's why. From a single run.
* **Lurkers in the Grotto's quest battle** (35): territory 252 is taken to have Middle La Noscea's Bg (so it loads
  `sea_s1f1`'s `QB_ManSea005_001`) from retail's positions; if its enemies don't appear, that's why. The boss's
  action 575 (every 6.4 s on the ally) has no action script, so it only auto-attacks, and the ally's 129 (on itself)
  is left out too. When the second wave comes (41 %) is read off a sparse health curve.
  Ul'dah's story stops after **Call of the Sea** (673), which hands over to Limsa Lominsa. The Ul'dahn Envoy's (528) later ticketer flights (Elyenora in sequences 4–255, L'nophlo in 8–255: a yes/no and an airship cutscene) send you to Limsa Lominsa's (POPRANGE1) or Gridania's (POPRANGE4) arrival from the client script's cutscene IDs only, unseen on retail; its POPRANGE2/3/5 are placed by order and retail's positions. Its Pugilist and
  Thaumaturge **Close to Home** (569, 570) come from their client scripts only: no retail capture, and their replay
  fixtures are 568's with their ITEM0 and ACTOR1 (ACTOR1's layout taken from the quest's todo location).
* Quest-spawned enemies take what brought them out from the current event's target (2026-10-27). For 3853's goobbue
  retail named the EVENTRANGE; whether Kawari's range events carry the range's layout ID as their target isn't
  checked. 307's layer doesn't set display flags 0x40003 as retail's spawns had.
* A search broken off (the bar interrupted) ends the event; retail plays the object's `..._EVENTACTIONNO` scene (one
  less than `..._EVENTACTIONOK`: Trial by Earth's 99) and no quest script here does (`questkit clientcheck` lists
  them as client scenes never played). Retail sent it right after the CancelCast and ActorControlSelf 138, with the
  EventFinish (quest 129's scene 8, capture c0a12d38); scripts would need a call for an interrupted bar to play it.
* Trial by Earth's tainted earth sprite cast Stone once in its only recorded fight: how often it casts is a guess
  (every 3 s), and its hitbox isn't recorded. Way of the Conjurer's fixture has some of the layout IDs Bob fought,
  not necessarily the ones that died.
* Like Mother, Like Daughter's quest battle (56): territory 291 is taken to have the South Shroud's Bg; the 80 % / 40 s
  and 65 % marks are from two runs (the 65 % from one: the lost run never got there); what brings the 1686 when the
  elementals haven't come is read as a 35 s timer from two spawns; the elementals' and the 1686's statuses (199, 17,
  9) last guessed times and do nothing; 1684's 863 has no effect scripted here; whom a conjurer heals is "the lowest
  below 40 %", retail's were at 11-74 %. Sylphie's own Battle talks come from her as an event NPC (kind 3); her
  standing there is a drop-in `event_npc` with `nonpop`, untested in game.
* Humanoid battle NPCs take their looks from BNpcBase's BNpcCustomize and NpcEquip, and for Trial by Wind's conjurer
  (base 1661) those gave a different person than retail drew (a Miqo'te, face 201, mouth 128: odd values); why isn't
  known (the row, the sheet's columns, or retail drawing him from elsewhere). The battle layers carry retail's
  `appearance` now; other humanoids (Yda, Papalymo, the Copperbell adventurers...) haven't been compared.
* QuestBattle row 54 doesn't name its quest (the server read 852182); a won battle falls back on the active quest that
  names it. Whether the other rows' Quest column is right was only seen through their battles working.
* Quest battles now take their ContentFinderCondition (level sync, ZoneInit); a quest battle that has none still
  syncs nobody. Kill EXP in them follows the duty rule (none while synced down), which retail's quest battles weren't
  checked for.
* Trial by Wind's and Trial by Water's quest battles (54, 16): territories 289 and 234 are taken to have the East
  Shroud's Bg from retail's positions (their NPCs are in `fst_f1f2`); if their NPCs don't appear, that's why. 54 has
  one run: its health marks (75/40/25 %), the conjurer's target order and its last arrival's kit (416, then 969 every
  3 s) come from it alone, and 416's status 57 lasts a guessed 30 s (unscripted). In 16 the lancers' 43 % yell line
  rests on five hits (43.0 % yelled, 43.2 % didn't), whom a lancer helps after his own yellowjacket falls (the
  nearest) is a guess, and the archer's Misery's End (103) is left out; the waves' "one enemy left" held in both
  runs.
* Trial by Water (92): the NPCs' lines are placed by the client script's text keys (the healer 13/14/21, the wounded
  Wood Wailer 9/15/20, Sylphie 18); which of the empty scenes (3, 4, 6, 11, 12, 16, 19, 22-33) belong to ACTOR4,
  ACTOR8-17 and EOBJECT1 isn't known, so those NPCs play nothing (as they'd show). ACTOR5, ACTOR6 and ACTOR18's
  layout IDs (3891735, 3816986, 3859524) are matched to them by their scenes, not by the game's LGB. The Cure's heal
  on the Wood Wailer isn't in Kawari's ActionEffect (retail's carried one).
* The mildewed goobbue's timeline (`MildewedGoobbue_355`) has no hitbox (none recorded) and casts on the lightning
  sprite's spacing, not retail's ~3.1 s.
* Monster timelines are keyed by BNpcBase, so 503's marshlights (base 187, name 901) fight as the coblyns that share
  the base (auto-attack 870 only; retail's marshlights also cast 516 once, about 6.5 s in). Needs per-name timelines.
* Duty, Honor, Country (551): territory 210 (w1e4) is untested in game (arriving at POPRANGE0, Owyne and Papashan there,
  leaving through the exit warp 0x2000D), as is EOBJECT0 (2001427) appearing in Central Thanalan in sequence 2; the
  client's (empty) scene 6, probably for a broken-off Waiting bar, is never played, and Kawari ends the event then.
* Quest battle 38: the sequence-4 trigger is a guess that fits all three runs; the second elite's falling line was never
  seen; the allies' Cure interval (8 s) is fitted; actions with no script are left out of the timelines (Rampart,
  Flash, Sword Oath, Foresight, Fracture, Bloodbath, Skull Sunder, Featherfoot, Second Wind, Shield Bash), so Garibald
  hits Papashan less often than on retail.
* Quest battle 38: Corrupted Tail (619) doesn't give Poison (retail: status 18, 30 s); the masked mage's spells come in
  its timeline's order from when it spawned, where retail's (Fire II, Fire, then Blizzard II ×4) starts when it's first
  hit; Thancred's Flash isn't used.
* Retail applies an action's damage when it hits (a player's cast about 0.57 s after its bar ends, though its
  ActionEffect comes 0.5 s before the bar ends); Kawari applies it at once, when its effect is sent (at the bar's end) and sends the target's UpdateHpMpTp just
  before the ActionEffect, so the HP bar can drop before the swing lands. (The result that follows, EffectResultBasic,
  is sent since 2026-10-21, see [Combat](combat.md).) Results for AoE targets other than the one shown aren't sent.
* A player's auto-attack shows animation lock 0.1, where retail sends the player's remaining lock (0.6 with a
  weaponskill, then 0.56, 0.11…, never below 0.1). Ranged classes' auto-attacks are action 7 at 25 yalms (Sapphire's;
  retail's Shot, 8, isn't in Bob's captures); damage types for classes other than Gladiator, Conjurer and White Mage
  come from their weapons; NPCs without a recorded hitbox count as 1.5 yalms for auto-attacks.
* PrepareZoning for quest moves goes out with hide_character 0 and flags 0; retail sent hide 1 with flags 1
  (HIDE_TERRITORY_NAME) for moves within a zone, 9 for airship flights and 8 for the warp NPCs (Bob's 2026-10-18
  capture, The Ul'dahn Envoy), so Kawari probably shows the territory name on in-zone loading screens. Every quest.
* NPC area attacks show their damage on the others they catch only as flying text (a one-target ActionEffect).
* Oh Captain, My Captain (quest battle 39): Fufulupa's Gut Rip (477; Critical Skill on himself before sequence 5) and
  Baldewyn's Rampart (10, once a run) have no action scripts, so neither is used (Heartstopper's recorded amounts are
  retail's critical hits, so its damage matches anyway); Fufulupa keeps his True Thrust, Feint and Leg Sweep rotation
  in the Heartstopper phase, where retail only auto-attacked (a rotation can't be switched off from Lua); the
  thaumaturge casts in a fixed order every 6 s, where retail used Blizzard II only with the player close; hitboxes are
  guessed (0.5). Quest 336's layout 3931797 is taken to be ACTOR1 (Baldewyn) from the client's script.
* Way Down in the Hole (320): in sequence 4 the client also offers ACTOR7–12 and EOBJECT0 with empty scenes (18–25);
  which scene goes with which isn't known, so the server just ends their events. The foreman's waiting scene (15) uses
  the usual talk flags; retail never played it.
* Event items: only the cast, the effect and the quest's event are done (the Casting condition with the cast, as
  retail's, `start_event_item`; a thrown one's too since 2026-11-11).
* Attuning to a main aetheryte outside a quest ends the event after the bar (`Aetheryte.lua`); inside Close to Home
  retail follows the bar with the aetheryte event's notice 0 (one argument, 0), which probably also gives the
  "attuned" message on its own, but a plain attunement hasn't been recorded to confirm it.
* Ul'dah's Close to Home: the client lets you talk to Momodi (ACTOR0) in sequence 1; its scene 7 only returns true and
  scene 6 is empty (as Gridania's 124 has 4 and 99). What retail does with it isn't known, so she just ends the event.
* A quest script without `onEnterTerritory` leaves an enter-territory event unfinished (the client waits). The fix (`02e22b5`) was rolled back with the first quest engine and not redone.
* Quest sequence/variable changes aren't logged or noted in packet captures any more (also lost in the rollback); use the captures' `UpdateQuest` packets or `//gm quest inspect`.
* `questkit replay` checks the scenes retail played and their flags (fixtures from Chronofoil), not what a scene
  shows: a scene the scaffolder guessed (marked TODO) can still be the wrong one with the right number. A progress bar
  broken off (retail then plays the "no" scene, e.g. 99 or 8) isn't replayed: Kawari's scripts get no callback for it.
  Unlocks sent as a quest completes are taken as the Quest sheet's (the server's `finish_quest`) and not checked.
* `questkit battle-sim` doesn't compare the NPCs' casts or yells, positions, enmity, or the player's health; one run
  per battle can't tell a health mark from a timer (Underneath the Sultantree's third pair, To Guard a Guardian's
  second wave, which came with five of the first six down rather than all six).
* The quest battle prompt is always allowed (`ServerRequestReply` result 0); retail presumably checks something.
* Unsolved Mystery's ambushers (BNpcBase 166, timeline `UnsolvedMysteryAmbusher_166`): one used action 426 on itself
  (status 204) about 2 s before it fell in Bob's capture b8432613; when and why isn't known (one use), so they never
  do. Their True Thrust's pace (8.8 s) is from three uses.
* Aetheryte tickets: only the Vesper Bay ticket's landing is recorded (the destination isn't read from its ItemAction
  Data, whose meaning isn't known). Item casts now come from the ItemAction's action; items with a cast there that
  retail uses instantly would now show one (none known).
* Gear coffers: retail's packets for opening one aren't recorded (Kawari gives the pieces as an event's items), nor
  what it says when the class has no set or the coffer is used in a duty (the client may refuse those itself). Only
  the Level 18 and 24 Weapon Coffers are filled in, from the wiki (whether a gladiator really gets the shield too is
  its listing's reading); the ward potions share the coffers' action and now say their contents aren't known.
* An Elezen bot's feet showed nothing in the glamour "keeper of the lilacs" (feet model 410, variant 1). Probably
  the same cause as the Hyur woman's Miqo'te legs (2026-10-27): the race rules weren't read at all (EquipRaceCategory
  in the wrong language), so nothing was left out; fixed, not yet seen in game. A glamour line whose name matches two
  items takes the later row, which may be the wrong one.
* The teleport crash of 2026-10-27 is put down to the retainer slots sent after dismissal ([Retainers](retainers.md));
  it was the one thing out of the ordinary in the packets before it, but the crash itself (inside the game) wasn't
  traced to that packet. If it comes back, its bundle and Dalamud crash log are wanted.
* Glamour plates: the second dye's bit (4) is a guess (only the first dye was seen), and what retail sends back for a
  dyed save beyond the plate isn't recorded (Kawari takes one dye and sends its slot.)
* Dressed to Deceive (574): which of each person's two later scenes is for being dressed isn't recorded (retail was
  played wearing the attire and played the first, 3, 6, 9, 12, 15); the other (4, 7, …) is taken as not dressed,
  with no progress. Amalj'aa Wrong Places' (622) other lines go by the order of the client's IsAcceptEvent.
* What Poor People Think's three paupers walk off and fade in their scenes (the client's WalkOut and Transparency);
  whether the server should take them away afterwards (they're the quest's listeners from sequence 2) isn't known, so
  it leaves them, as the client shows them.
* A Proper Burial's other lines (Marques' scenes 4, 7, 13 and 17, Ourcen's 18) go by the order in which the client's
  IsAcceptEvent offers each sequence's NPCs; retail played none of them, so which line goes with which isn't checked.
* For the Children's zombie and zombie mage (timelines `Zombie_18`, `ZombieMage_19`) were only seen at level 13. The
  same bases in Eastern Thanalan's open world are level 17 and never fought in the capture: the zombie's auto-attack is
  scaled from level 13, the mage's Thunder isn't (13 for every level). The zombie's Heartstopper (866, once, on no one)
  isn't modelled.
* Lord of the Inferno (343): its notice 3 is sent as the player leaves the cleared Bowl of Embers (`onDutyCleared`);
  retail sent it in the duty as it was won. Its others' lines in sequences 2-4 go by the order of the client's IsAcceptEvent (retail
  played none).
* The MSQ quests from captures 412ba91d, b166797f and 7d85c539 (783, 3863, 3864, 787, 786, 799, 800; 2026-11-01, three characters) are untested in game. Terror at Fallgourd's (3863) scene for a wrong emote (6) wasn't seen in any run: its flags are a guess (HIDE_HOTBAR), and only retail's emote 35 is taken as the right one. Seeing Eye to Winged Eye's (786) enemy put status 18 on the player at 16 in two runs and at 12 in the third (a level 32 character, the others 33); its 330 isn't scripted beyond the status. Power of Deduction's (799) violated corpse (EOBJECT0, empty scenes 5 and 6) isn't played.
* Retail's quest journal keeps each quest in its slot (UpdateQuest's index): finishing one empties its slot, those after it don't move up, and the next quest accepted takes the first empty one (Bob's Chronofoil captures of 2026-11-01: 786 stayed in slot 1 after 787, in slot 0, was finished). Kawari's journal is a list that closes up, and sends it all again after every finished quest.
* Finishing one of two main scenario quests underway together (786 and 787) sent, where ScenarioGuide comes after the others, an 8-byte packet with AcceptQuest's opcode: the one still underway, then the one finished (`787, 786`, or `786, 787` the other way round; all three runs). What it does isn't known, and Kawari sends its ScenarioGuide.
* Key items thrown on the ground (2026-11-01, Rock of Rancor) are untested in game. How far from the spot the object can
  be is the EffectRange of the EventItem's Action (1276 for the firesand), which wasn't checked against the sheet (no
  game data where it was written; 2 yalms if it's 0): the server log says the radius and what it landed on at each
  throw ("Key item … landed at …"). Retail's Condition 4 (casting) during the bar is sent since 2026-11-11 (test
  `a_thrown_key_item_s_bar_is_a_cast`); whether moving cancels the bar wasn't seen (Kawari doesn't).
* Line of sight (`Collision::sees`, since 2026-11-02) goes from 1.5 yalms over each one's feet (a guess; retail's
  height and whether its enemies need sight for every action aren't known), to the target's middle whatever its size,
  and isn't checked for an NPC's sight aggro (`can_notice`) or for areas. Untested in game.
* Collision that's a simple shape (`kawari-navimesh`, since 2026-11-02) is placed as vnavmesh places it and checked
  against Haukke Manor's `bg.lgb` only (three walls, by their numbers); untested in game anywhere. Every zone's navmesh
  and collision are made again the first time it's entered after the update (collision version 4), which takes a while
  for open-world zones. What the shapes' materials mean (Haukke Manor's are 0x1000-0x17005, mask 0xFFFFFFFF) isn't
  known: all of them are kept, as walls and floors, so a shape that's only for the camera or for flying would stop
  walkers wrongly.
* `kawari-navimesh` can't make the Waking Sands' navmesh (territory 212, `wil_w1/evt/w1e6`): its level's bg path is empty, so it looks for `/collision/list.pcb` and panics (Bob's bundle 20261101-211113). Only the tool stops, run again at each visit; the world server goes on without a navmesh there (no enemies to move).
* Paralysis froze the player whenever a proc cut a cast short (2026-11-04, six bundles): the client waits after a
  proc for the server's next conditions, which retail sent 0.98 s after every proc and Kawari never did
  ([Combat](combat.md#paralysis-serverparalysisrs)). Now sent a second after each proc on a player, untested in game.
  A player in the middle of a progress bar gets the same; other statuses that may hold the client the same way (sleep,
  stun) weren't checked. Statuses whose param the scripts took from an ActionEffect's second byte may be wrong the
  way Paralysis's was (QB56Sprite_00313's 199 at 50, TrialByWindAdd_00416's 57 at 50, AcornArmageddon_00863's Heavy
  at 50): not checked. A cast cancelled by the player's death or a zone change leaves the Casting condition to the
  next condition update.
* Paralysis's durations came down 30, 15, 7 s as it was put on the same actor again (and once 3), which Kawari doesn't do; its chance (one roll in two) is the same whatever gave it. Whether a damage-over-time tick interrupts a progress bar wasn't seen.
* The quests from captures 9df4ff01 and 3a705e18 (743-763, 1104; 2026-10-30) are untested in game. So are 765 and 774-776 (capture a0149e5e, 2026-10-31) and 777, 778 and 782 (capture 20ef0e90, 2026-10-31; replayed against two more characters' runs, captures 9faa9bac and bd51ca23: the same steps). Big Trouble in Little Ala Mhigo's (782) bystanders in sequence 3 (ACTOR6-10, which the client accepts talks with) have no scenes: which of its scenes 14-20 belong to whom isn't known, and retail's talks to them weren't recorded; Bertliana's (9, 12, 21) flags are a guess (0x2000). In Wilred Wants You (778) ENEMY0 fell first in all three runs; talking to Wilred again after one has fallen brings it out again (either counts, as the client's script accepts both while UI8AL is below 2; what retail does then wasn't seen); He Ain't Heavy's enemy's 405 (a placed cast) takes its shape from the Action sheet, unchecked, and its 130 is one hit of two. Believe in Your Sylph's lookout
  emote relies on the client's StartEmoteEvent (fork opcode 401): the server doesn't check the player is in
  EVENTRANGE0. Its empty scenes (2, 4-6, 9-11) and Highbridge Times' and Tea for Three's other lines aren't played or
  go by the client's IsAnnounce order (retail played none). Where There Is Smoke's dusk bat's 475 (32 on the player)
  and its 181 self-heal (57; Kawari's 57 heals a fixed 52) aren't used. A Pup No Longer (1104) unlocks InstanceContent
  40001, retail's number; the Twin Adders' and Immortal Flames' versions aren't scripted.

## Central Thanalan

* Some of its monsters still weren't fought on retail: the Sun Bat, Spriggans, Cochineal Cactuar, Antling Worker,
  Amalj'aa, Qiqirn Scrambler and the guards deal the generic 12 %, and the numbers that are there are from a
  gladiator (a tank). Skills seen but not scripted: the Huge Hornet's Sharp Sting, and those listed in combat.md
  ("Open-world spawns"). Only the Star Marmot's and Huge Hornet's drops are in `Drops.lua`.
* Western Thanalan (140) has retail's spawn points from one walk around it (2026-10-18), so spots Bob didn't pass
  near are missing; most of its kinds deal the generic 12 % (damage was recorded for six), none of their skills are
  scripted, and only the Sun Midge Swarm's drop is in `Drops.lua`. Not tested in game yet.
* Ovjang (the hunt mark) stands at one of its spawn points; retail moves it between them.

## NPC conversations (Script Check)

* 34 of the 94 NPC and opening scripts leave client scenes unplayed (Script Check's "worth reading", 2026-10-27):
  mostly systems not built (Grand Company officer 1-10, retainer vocate 5-55, Wondrous Tails, Free Company, Triple
  Triad tournaments, the bicolor gemstone vendors' shops); see [questkit](questkit.md#clientcheck).

## Hunting Log

* Built 2026-10-27 ([Hunting Log](hunting-log.md)): finishing a rank, the Grand Companies' logs, party kills and 543's
  second number are guesses or not done (see its page). Auracite (Kawari) backups and ArchonEngine ControlPanel don't
  carry it.

## Achievement Certificates

* One certificate per 50 achievement points is the wiki's number, not checked against a retail character's points.
* When retail plays Jonathas' greeting (scene 0) isn't recorded; Kawari plays it to someone never given a certificate.
* The nested shop's five scene parameters (`0xF8380000, 248, 0, 0, 1`) aren't sent or understood.
* Auracite (Kawari) backups and ArchonEngine ControlPanel don't carry `player_currencies` (the certificate counts).

## Marketboard

* Retail refuses a search started a second after the last (error 0x70000001); Kawari answers every one. The u16
  after each item's count in a search reply is unknown (sent as 0). The search box's text isn't used.
* The purchase opcodes (MarketBoardPurchaseHandler 258, MarketBoardPurchase 610) are FFXIVOpcodes', not yet seen
  working on Kawari; the first purchase tells. They have no finders for the startup scan.
* Only Gridania's marketboard object has a script (`CmnDefMarketBoardGridania_00039`); whether the other cities'
  boards are other CustomTalk scripts hasn't been checked.
* What the search request's flag byte means (1 on the Weapons and Armor tabs, 0 on Items and Housing) isn't known;
  it's ignored, and every item of the category is listed.
* Searching by name isn't handled by the server: the captures had none, and the client seems to do it from its own
  item list, then open the item the usual way. Untested.
* Retail's order of a category's items isn't known (it isn't item level or ID); Kawari sends them by ID.
* A purchase doesn't send retail's inventory transaction for the gil, only the gil and item slots (as the NPC shops
  do); the upper 16 bits of a search result's count (1, 2 or 5 on retail) aren't known.
* Selling isn't there: players can put items up for sale with a [retainer](retainers.md), but those don't show on the
  board.

## Retainers

* Calling a retainer and their menu were tried on Kawari (2026-10-18 and 2026-10-20 bundles). The 7.57h1 opcodes are
  from the executable, not yet seen from the client except the marketboard's.
* The vocate's menu choices other than hiring and letting go (5 and 8 on retail) end the visit; hiring's yield 3 is
  answered with one 0, not retail's eight zeros on resume opcode 329 (upstream's EventResume8 is 513: which is right
  for 2026.10.06 isn't settled.)
* Approval to hire retainers (2026-10-26) comes with The Scions of the Seventh Dawn's unlock link 3 (granted from
  `RECORDED_QUEST_UNLOCKS` since its Quest row has no `REWARD…` parameter; until then the bell refused everyone). Whether retail
  also wants An Ill-Conceived Venture finished first (the vocate's own quest, which Kawari doesn't have) isn't
  recorded: no capture has a bell or vocate between the two. The vocate's answer before it (scene 3) is a guess from
  its scene's name.
* Partial gil withdrawals weren't captured: Kawari treats a 0 amount as "everything" and any other amount as an
  entrust.
* Retail sends a retainer's containers two or three times with an Equip in between, and ActorControlSelf 570/574 before
  their menu; Kawari sends the containers once. Scene 2's third number is guessed (5 − retainers hired).
* The retainer's spot at a bell (`Zone::retainer_spot`: the bell's bound pop range, or the nearest) is untested against
  the game's layout; bells inside shared groups wouldn't be found, and the retainer then stands in front of the
  player as before. See [Retainers](retainers.md). A new retainer is level 1 (retail 0) so the NPC has stats.
* Items on the market don't show on the marketboard and other players can't buy them; listings never end
  (PlayerRetainerInfo's time is 0, retail sends about a week ahead.) No ventures, no class change at the bell, no sale
  history (sent empty.) Withdrawn equipment goes to the bags (retail put a weapon in the Armoury Chest.)
* The lobby's character list doesn't show retainers (DistRetainerInfo is still empty).

## Damage formula (2026-11-10)

* Untested in game: the formula was fitted to retail captures ([Combat](combat.md)); Kawari's own damage wasn't
  checked against it with Bob's game data. Up to 50 it fits within 1-2 %; levels 51-79 and 81-89 between the anchors,
  and tanks past 50, are interpolated from the community's numbers, not recorded. Party bots use it too, so past 50
  they hit about three times as hard as before.
* Enemies more than seven levels above the player take what seven above do (0.81): further up wasn't recorded.
* Scaling added on 2026-11-12 ([Combat](combat.md)) that the captures can't check: Tenacity's and speed's coefficients
  (the community's 100 and 130), food's bonuses (no meal in the captures without a class change) and whether food or
  the party bonus comes first; materia's per-item cap (the captures' materia were all well under it). Bots get neither
  the party bonus nor food. Defense is taken against the recordings' characters' 8 % and 15 %, a median: their
  under-geared stretches (3-4 % at 24 and 28-29) recorded hits a little higher than retail's for someone geared.
* The Arcanist's Ruin at levels 3-5 did 0.79 of the formula (three groups; 8-45 fit), and enemies vary a little among
  themselves on retail (about ±3 % at the same level), which Kawari doesn't model.
* A quest battle's DPS status (1265) was only seen on an Arcanist; melee and physical ranged players get it too in
  Kawari. What it does isn't known (not damage). Whether the role statuses change damage taken isn't checked.

## Combat timing

* **Limit Break** isn't there. Retail's gauge is ActorControlSelf 505 (bars, amount, 10000 a bar, reason, 3 in Trust
  duties, 0): filled every 3 s in combat by 220 with 3 bars (8 players), 75 with 1 bar and 180 with 2 (4 players: 2
  bars only for the final boss), 140/220 in alliance raids (2 bars on trash, 3 for bosses, emptied at each boss);
  using any Limit Break empties it. Kawari's ActorControl table has 504 twice (SetStatusIcon and a SetLimitBreak) and
  a 4-value 505.
* Enmity: party members aren't put on an enemy's list at 0 when one of them engages it; Provoke adds 1 above the top
  (retail adds thousands); Shirk (25 % of the user's enmity to a party member) isn't done.
* Ground-targeted actions don't place anything (Asylum, Sacred Soil...). ZoneInit's flags in duties are 0x22 (+0x40
  for the recorder) where retail's are always 0x2A, and flying is allowed everywhere outside duties. Retail sends
  ActorControlSelf 1530 and 943 after the player's own spawn, and mounting's packets 0.58 s after the action (Kawari:
  together). StatusEffectList's header bytes 2 and 3 are the real and the synced level on retail.

* Over-time ticks aren't summed per target or put on the target's own phase, aren't scaled by Spell Speed and never
  crit; Lucid Dreaming's MP tick sends no packet.
* Presence of Mind and slows don't change PlayerStats' haste (so the GCD's recast isn't shortened, only casts); item
  animation locks are 0.6 s. Haste isn't something the client works out: retail's PlayerStats sends 85 under Haste
  (status 1091) and 160 under Slow (9), numbers Status.ParamModifier (−10, 15) doesn't give.
* Kawari doesn't keep recasts itself, so it never refuses an action that comes too early (retail answers 582 "Not
  yet ready.", ActorControlSelf 700, with the elapsed time), and it acts on a request that arrives during a cast
  straight away (a second cast task beside the first), where retail answered one 1.4 s after it arrived, with the
  previous cast still on (Bob's Chronofoil captures). Black Mage's Astral Fire and Umbral Ice don't change MP costs,
  and there's no Recitation to make Scholar's heals free.
* Recasts of spells and weaponskills outside the GCD group (Drill, Gnashing Fang's own timer...) are sped up by Skill
  or Spell Speed as the client's GetAdjustedRecastTime does, but no retail capture has one to check the server's number
  against.
* Raise: Weakness and Brink of Death don't lower stats; status 418 isn't taken away on the first action; the stand-up
  doesn't wait the 3 s retail does; Weakness doesn't survive a death.
* Statuses: Temperance's healing bonus, the 5 s party auras (Temperance 1873, Asylum 1912, the bard songs), fixed status slots (retail
  keeps a status in its slot and leaves gaps; Kawari shifts them, so EffectResult indexes can be off), removed-status
  entries (retail: slot, status 0, source 0xE0000000), Arm's Length's Slow on attackers, and enemies' debuffs from
  Bob's captures (scan `enemy_statuses.csv`) aren't done.

## Duty clear rewards and enemies' gil (2026-10-28)

* Untested in game. The cleared bitmasks are indexed as upstream's unlocked ones (InstanceContent ID less 1, 10001,
  20001 or 30001), which isn't checked against the client: if it's wrong the Duty Finder shows the wrong duties as
  cleared. Characters that cleared duties before 2026-10-28 have none recorded, so each one's next clear counts as its
  first (the clear EXP and the bonus once more).
* The duties' boss EXP (FinalBossExp, BossExp: retail's 19,386 for Sastasha's last boss) isn't given; bosses give the
  usual kill EXP.
* Enemies' gil is fitted to levels 15-17 only (uniform, so level 17's 9 gil comes about a fifth as often as on retail); whether higher-level duties' enemies follow the same factor isn't
  recorded. Which enemies drop is "spawned with the instance" in Kawari, which matched retail's dungeons' trash in the
  three dungeons; a duty whose script spawns its trash itself would drop nothing. Each player rolls their own amount
  (retail's other players' drops aren't in the captures). Open-world enemies give none (retail: three drops of 1 gil
  in 46 field kills, not modelled).
* Synced-down players get the clear EXP and the bonus as anyone does (retail's clears were all at the duty's level).

## Coffers and loot

* The wiki's loot lists are only in Sastasha, the Tam-Tara Deepcroft and the Copperbell Mines; their fixed-rate extras (a Triple Triad card, an orchestrion material) aren't given, and how many items a never-recorded coffer holds is taken as one.
* Untested in game (2026-10-22): opening, the rolls, the winner's item, the coffer fading out. LootMessages name bots
  by their actor ID; whether the client finds their names (they're in the party list) isn't known. Duty Support's
  personal loot (2026-10-28) is untested in game, and a full inventory then isn't recorded (Kawari says it couldn't
  add the item, and it's gone). Personal loot whenever only one player is there (Bob's rule, 2026-10-28: a party of
  bots too) is untested in game; retail's only one-player runs were Duty Support.
* A Greed or Need on a unique item the player has already isn't refused (retail answers 8). LootItem's `bonus` (set
  on retail's Aetherial Brass Wristlets, perhaps their random stats) is left at zero.
* Sastasha's coffers 98 and 100, the Tam-Tara Deepcroft's 15 and 119 and the Copperbell Mines' 59 and 65 were never
  opened in a capture and hold what their wiki lists say. An opened coffer coming back into range with some of its items
  decided isn't recorded (each of retail's 41 had none decided); Kawari lists them all.

## Party bots

* The glamour list (`glamours.txt`) is untested in game (2026-10-23): whether the client shows an NPC's glamoured
  models and both dyes as a player's. Items limited to a race or a sex aren't checked, nor whether a glamour's items
  are ones its classes could wear.
* The Glamour Collector's page reader follows Eorzea Collection's markup as of 2026-10-23 (checked on 21 live pages,
  not yet through the installed extension); a site change will need it matched again. A glamour listing more than
  one job in "Intended for" wasn't seen, so whether those come as several icons is unknown.
* The glamour API (ArchonEngine ControlPanel 2.21.0, `UI-Redesign` only) and the extension's ArchonEngine ControlPanel
  connection were checked against a test ArchonEngine ControlPanel in the workspace, not yet on Bob's server over the
  LAN or Tailscale. The server needs a build with this commit's `kawari-questdump classes` for the class list, and this
  commit's `KAWARI_DATA` path for the bots to see the list ArchonEngine ControlPanel writes.
* The tank's spot across from the party and its keeping up with the player are untested in game.
* Leaving out gear and glamour pieces a bot's race or sex may not wear (EquipRaceCategory, 2026-10-23) is untested in
  game.
* Hiding a Viera's or Hrothgar's hat when its model has no file for their race (2026-10-23) is untested in game, as is
  whether a hat they can show always has its own model file (a shared one would be hidden wrongly).
* Raising a dead bot and its getting up by itself after 30 s are untested in game (2026-10-23): whether the client
  shows a bot raised by the ZoneIn raise animation as it shows a raised player, and that a bot spawned again beside
  you stands (not dead). Healer bots casting Raise (2026-10-23) is untested in game; they don't walk to someone dead out of range
  (beyond 30 yalms), nor use Swiftcast (a role action, not in bots' kits) for it.
* Bots of every race (2026-10-23) are untested in game. How CharaMakeType's menus are read is from the sheet's layout,
  not checked against the game's data: that `Customize` is the look's byte, that the face's and ears' options are
  their values (or icons, then counted), and that a hairstyle option is a CharaMakeCustomize row. A bot with a wrong
  face or hairstyle (bald, or the default face) would show it.
* Bots shown as players work in game (Bob, 2026-10-23). Still to check: whether the party list now follows their HP,
  and what the client does with search info and the adventurer plate, which get no answer for a bot.
* Kicking a bot, leaving the party, Examine on a bot, tells to bots and bots kept as friends (the friend list's
  entries, calling one from it) are untested in game (2026-10-23); inviting one works (Bob). Kicking did nothing for
  Bob before this commit with no bundle to show why: the bot is now found by name too, and the server logs a kick
  that matches none. What the friend list's entry needs for the client to offer Invite to Party (online, same world,
  the context menu flag) is a guess from how Kawari lists players.
  Examine on a real player shows only what their spawn has (no items or stats: those are on their own connection).
  Retail's packets on the inviter's side when someone accepts weren't in the captures (only Bob being invited), so a
  bot's acceptance is sent as Kawari sends a player's.

## Sastasha

* **Its keys as key items** (2026-11-13, `questkit autoduty`): AutoDuty's route waits at the Captain's Quarters and
  the Waverider Gate until the party has key items 2000512 and 2000513, so retail gives them when the keys are taken;
  `s1d1_re.lua` only remembers that the key was taken (`has_captains_quarters_key` and the like) and gives nothing.
  How retail gives and takes them (the packets) needs one of Bob's Sastasha captures of 2026-10-22 (f4316611) or
  2026-10-23 (700cc294), which aren't in the release's `chronofoil.zip`.

* Bosses' timelines (Chopper, the quarters' reaver, both captains, Denn, Denn's adds) are from six runs' medians
  (2026-10-22/23); their actions that did no damage (351, 1000) are only animations, and when the bosses first use
  each action isn't recorded (Denn's 556 comes about 14 s in, retail's 18). Denn's adds come by time from his first
  target (47 and 63 s), which fits the Duty Support run; retail's trigger isn't known (one Duty Finder run had its
  wave 71 s after he appeared).
* The second captain's gate fight and the clams' loop are untested in game (2026-10-23). Whether the four corsairs are
  in Sastasha's layout as Kawari reads it isn't known (Bob's earlier Kawari run never spawned them); if not, the
  server logs that it couldn't find them. ActorControl 503 and 11 (retail's, putting them on the enemy list) aren't
  sent.
* Boss Mod lists action 835 (no cast, on a player) for the last boss's adds (BNpcBase 418); `DennAdd_418.json` never
  uses it (`questkit sources --audit`, 2026-11-10). Not looked for in a capture yet.

## The Tam-Tara Deepcroft

* Untested in game (2026-10-24). Scripted from three retail runs (see [Dungeons](dungeons.md)).
* No shortcut (its pop ranges aren't known without the game's layout files); log 2008 near the key isn't sent.
* The gate (2000060) opens through the generic GimmickAccessor prompt; the third room's pack is held unaware until
  then. Whether players can hit it through the gate, and whether Kawari's NPCs path around the rooms' walls, isn't
  checked.
* The rooms' bosses' 912 and the last boss's statuses (208, 17) are only applied, not scripted; its drain gives back
  a quarter of the hit (retail 5-10 of 22-40), against players all of it (action.rs's drain rule).
* The timelines are by BNpcBase, so other places using bases 76, 82, 83, 125-127, 137, 567 or 1345-1388 (if any) get
  them too.
* Coffers 15 and 119 were never opened on retail and hold one of the others' items.
* The shields' tether to Galvanth comes with their spawn only (as retail's); a shield spawned before the boss would
  have no tether (the script spawns them after it).

## The Copperbell Mines

* Played through in game by Bob (2026-10-25, bundles); the fixes since (Kottos' leaps, the second and last bosses'
  helpers, the ambushes' appearing, the Duty Information counts, the entrance facing) are untested in game.
* The dropped items (2000159, 2001531, 2000179) are spawned with `spawn_eobj`, which needs them in the duty's layout
  (retail spawned them with layout ID 0); if they aren't, the server logs it and the way stays shut.
* 28462's and 28760's effect (`kk 00 1b 00 00 00 00 80` then the action ID, `kk` per target) isn't decoded or sent.
  It isn't the knockback: Rive (The Lominsan Way) had the same entry after its knockback, which is kind 31
  ([Combat](combat.md#enemy-damage-to-players)). Retail's helpers' SetTarget on the player isn't sent. 28544's and 28549's damage
  are stand-ins (28546's): nobody was caught by them in Bob's three runs.
* The second boss's 14449 helpers are hostile (display flags 0x60008): held, they don't move or fight back, but
  players and bots may still be able to target and hit them.
* The last hall's trigger is a box around where retail's started (z below 55, y below -50) checked against the first
  player only (`info.player`).
* Statuses 8, 18, 53, 203 are only applied or nothing; the shortcut and the end's log 9393 aren't sent.
* Not done from the audit of 2026-10-25: the Limit Break gauge (retail's ActorControlSelf 505, 116 in the first
  stretch; Kawari never fills it); ActorControl 13 (3, value), sent from the player every ~6 s (its handler isn't
  identified); the second gigas pair (2790237, 4023509) pulled 5 s apart where retail linked them.
* Duty clear doesn't send retail's completion EXP and gil logs (9393 with 24200, 2163 with 2700), ActorControlSelf 130 (3, 1)
  and 257 ×3, or UnkContentFinder2 (3, 4); the clear rewards need checking against the InstanceContent sheet.
* The timelines are by BNpcBase (1395-1403, 1421, 14448, 14449, 14460, 14537, 9020).

## The Thousand Maws of Toto-Rak

* Untested in game (2026-10-30), from three runs (one with Duty Support, two Duty Finder runs). The director was
  checked against all three with a throwaway lupa harness (its webs, objectives, rooms, adds, eggs, tail and end within a tick of
  retail's where the run's own positions and health allow); no questkit command does that for a duty yet.
* The drop-in's LGB (`bg/ffxiv/fst_f1/rad/f1r1/level/planmap.lgb`) is the TerritoryType's folder plus `planmap.lgb`, not
  checked against the LVB. The last boss's tail has a layout ID of the fork's (99000266); retail's had none.
* The webs' collision: party NPCs walk through a web after 2 s held up by it (as any wall in a duty), burnt or not;
  they follow the player, who can't pass one standing.
* Not done: the shortcut, the event object near the last boss 4 s into each 702, the item given as the duty was
  complete, what ActorControl 30 and 31 on the last boss do, the boss's facing for its tail. The last boss's 704 comes
  at once at 55 %; retail's waited for what the boss was doing (up to 3 s). The poisons' ticks are retail's numbers;
  how often they tick (3 s) and their 15 s are from the StatusEffectList, and 1355's and 1360's 345 and 364 were seen
  on few targets. 346 (1359) reached nobody and does nothing.
* Duty Support's Lancer (41) at 24 has a guessed 639 HP.
* The slime's status 67 (2026-10-30) is untested in game: whether the slime's PCB material is its own (not the
  floor's around it) can only be seen on a server with the game (the log names the materials, or says the floor was
  left out). Party NPCs don't get it (whether retail's Duty Support members did wasn't recorded). The first entry
  into Toto-Rak after this update makes its navmesh again (a minute or so) for the materials.

## Haukke Manor (2026-11-02)

* Untested in game, from four runs (one with Duty Support, three Duty Finder runs). The director was checked against
  the Duty Support run's steps with a throwaway lupa harness (its variables after every key and door, the rooms, the
  drops, the last fight's adds); no questkit command does that for a duty yet.
* The script's name (`f1d2_re`, the ShortCode) is by analogy, not checked against the sheet: if the server log says
  `content/f1d2_re.lua` isn't found, the director has another name.
* The way back up from the cellar (2012969) is taken to be a shortcut GimmickAccessor with its own pop range and line
  (Param1, Param2); if not, the script sends the party to pop range 9228207 itself. Which it is shows in the log only.
* The keys 2000306-2000308 and the doors 2000331, 2000339 and 2000345 weren't used on retail; they're taken to work as
  the others of their series. The stairs' locked line comes after a prompt here (retail: no prompt, the gimmick's own
  log 2004 and EventFinish result 0x65).
* The shortcut outside the last room (11438019, from its cutscene) isn't seen on retail. 2000324 (the first boss's key) is spawned with its layout ID (retail: 0).
* Duty Support's Lancer (41) at 28 has a guessed 779 HP. Not done: what the last boss's 28640, 28644 and 28649 and the
  eyes' gazes (513, 29709, 29710) do; wiki loot pools.
* **The sunken space by the west wing** (the ground floor at x -34 to -18, z 16 to 44, at -3.2, with a strip down to the
  cellar at z 36): its edge at z 45 is closed by invisible walls whose collision is a simple shape (BgParts 3819851,
  3819852 and 3867912: boxes 5.4 and 3.9 yalms high), which `kawari-navimesh` left out before 2026-11-02. They're in the
  collision now (version 4: [Combat: walking on the collision](combat.md#walking-on-the-collision)); untested in game.
  Bob's next run (bundle 20261103-051833, version 4) still had bots and enemies down there: they walked on past the
  shelf (3867912) after their patience, onto the sunken floor; they no longer go on to a floor deeper than they'd step
  down. The sunken floor still has gaps in its edge (x -34 to -32, -31 to -26, -21 to -14 at z 45) with nothing in
  Kawari's collision; whether retail has more there wasn't checked with the zone's shared groups.
* Retail's clear of a story dungeon put the player at the dungeon's door (the quest's POPRANGE0) for Skeletons in Her
  Closet and Into the Beast's Maw, whose next step is a cutscene in that zone; other quests with a dungeon and a
  POPRANGE0 weren't checked. Kawari's ZoneInit there doesn't carry the pop range's ID (retail's did); the positions are
  the captures'.
* Lady Amandine's fight (since 2026-11-03) is untested in game. Kawari doesn't send retail's opcodes 820 (as Beguiling
  Mist walks a player), 839 and UpdateClassInfo (as it ends); what ActorControl 82 (1132) shows isn't known; the eye's
  gaze comes from the director's half-second ticks (to 0.5 s). 28645, 28646 and 14506's actions are as before (their
  effects from the recordings, not looked at again.)
* The first boss's 705: the Duty Support run's first came 99 s into the fight, the Duty Finder runs' 22-27 s in; the
  timeline has the first.
* Haukke Manor's eye at the last room's door (since 2026-11-03): untested in game. Kawari doesn't send retail's
  opcode 942 `[the eye, 1]` (unknown), doesn't put the eye's health back to full as it flies (retail's did, over a
  second), and doesn't send ToggleWeapon off and SetBattle 0 at once (the eye and the party leave battle on their own as
  its enmity goes).
* Haukke Manor's doors: untested since the open ones are left open (collision version 5). That a door placed `Open`
  leaves out only its collision boxes is from the doorways' boxes in Kawari's collision (two faces 0.225 apart, 4.6
  wide, 10.5 high, with a box's material 0) and the layout's door states, not from the door's `.sgb` (not seen): a door
  whose closed collision is a mesh would still be shut.
* With party bots (a Duty Finder run, 2026-11-02): the tank and three enemies ended in the cellar under the player,
  whom the enemies' area attacks there hit through the floor, and he died. Fixed as above (area attacks' height, the
  path's drop, the doors sealed); untested in game.

## Brayflox's Longstop (2026-11-03)

* Untested in game, from two runs (one with Duty Support, one through the Duty Finder: Bob's captures 44df02fc and
  c9145fdc). The director was checked against the runs' steps with a throwaway lupa harness (its variables after the
  key, the doors and each boss, the waves, the ambushes, the dragon, the puddles).
* The dragon killing the third boss (2026-11-10) is untested in game, and the kill is credited as any death the
  director causes; retail's Defeated named a party member (a Duty Support member, the one fighting it).
* The shortcut (2000700) takes the party to the layout's pop range outside the room of the boss last pulled
  (11438223-11438226): nobody used it in either run, so that order is the layout's and the other dungeons'. The
  puddles' map effects are found by their SharedGroups in ContentDirectorManagedSG (`map_effect_at`): that its layout
  IDs are the LGB's SharedGroup instance IDs is taken from Kawari's reading of the sheet, not seen; if not, the
  indices seen on retail are used, which spot is which within a set still a guess then. The event ranges' boxes are
  copied from the layout (Kawari's map ranges ignore a box's turn: the script checks the turned box itself).
* From one or two runs: the first boss's waves after the third (the same spots again), the dragon's leaving below 71 %
  (and, if nobody wears it down, after 24.2 s: a guess from the longest seen), what 505, 506, 508, 448, 498, 29618 and
  28660 do to whoever they catch (no one was caught in either run: a plain hit for now), 982's status 60 (how long it
  lasts wasn't seen: 15 s), 991 Bravado (no effect seen), the trash's rotations for actions used once or twice (364,
  365, 498, 1010: 20-30 s guesses).
* The event NPCs after the clear come whenever the party had NPCs in it (Duty Support members or party bots): the
  director can't tell the two apart.
* Not sent: the logs 9393 and 2163 at the clear, DirectorSetupMapEffects64 at the
  start, opcodes 820 and 664 (unknown).
* Duty Support's Lancer (45) at 32 has a guessed 1120 HP; the members' kits at 32 beyond the Marauder's combo and burst
  are the trials' (see [Party NPCs](party-npcs.md)).
* Quest 832 moved to sequence 4 on retail as the player used the exit (EventNotice2 and UpdateQuest just before the
  PrepareZoning), not at the clear; the player came back where they had queued (both runs). Scripted since
  2026-11-03 (`onDutyCleared`), untested.

## The Bowl of Embers

* Untested in game (2026-10-27), as is the party wipe (`server/wipe.rs`), which now plays out in every InstanceContent
  (the dungeons too) when every player is down. In a dungeon it sends the party back to the start but leaves the
  enemies as they are (they lose their targets and go home); retail's dungeon wipes weren't checked.
* What happens when the Nail isn't killed is Cactbot's (Hellfire 44.3 s after the 50 % line); the first attempt wiped
  before. Hellfire's damage then is the killed-Nail one.
* Ifrit's hitbox (3.5) is a guess from the members' distances. Eruption goes under anyone of the party (retail's nine
  were never on the player). Retail's helpers cast at a point on the ground; Kawari's are put there and cast on
  themselves. Kawari spawns nine helpers where retail spawned twenty. BNpcBase 401's timeline (`IfritHelper_401`)
  applies to any 401 elsewhere.
* The members spawn on the player (retail's a step ahead in a wedge); the Thaumaturge's and Lancer's pull lines are
  taken as first attempt and later ones (one of each seen). Status 775 (Ifrit's during Hellfire) is only applied, with
  `set_invulnerable`; whether retail makes him untargetable then isn't recorded. The clear's logs aren't sent.

## A duty's music (2026-11-11)

* The opening scene's first parameter is the duty's InstanceContent BGM (`dungeons.md`); untested in game, and not
  checked against the game's sheet here (no game install): retail's numbers are 155 (the Howling Eye), 157 (the Navel)
  and 0 (the Stone Vigil). If a duty plays the wrong music, the column is the wrong one.

## The Navel (2026-11-06)

* Untested in game, from three runs that each won at the first attempt (Bob's Chronofoil captures 650dbfd8, 0fd8bce0,
  48aaf6fe): a wipe goes as the Bowl of Embers' (`onPartyWipe` brings Titan back), not seen on retail here; the
  Marauder's pull line is said at the first pull only.
* The Heart left standing: Cactbot's "Enrage" at 73.6 s after the second Geocrush began; Kawari takes the Heart away
  then and Earthen Fury comes as after its fall, with the broken-Heart damage. What retail's does (and whether Earthen
  Fury's damage grows with the Heart's health) isn't recorded.
* The leaps come 1 s after his health is below 80 or 50 % (2.4 s after a cast's end while he casts); retail's were 0.9
  to 3.3 s after, so the rule is a fit. Titan's hitbox (3.7) is a guess (the Duty Support Marauder's 6.7 yalms against
  Ifrit's 6.5 for his 3.5). Geocrush's fall-off is a line through 17 hits at two distances (4.6-4.8 and 18.8-22.1
  yalms); nothing between was recorded.
* Status 292 (Granite Gaol) never ends by itself (retail's gaols were broken in 11-22 s; its own time isn't known).
  It stops a gaoled NPC acting, and since 2026-11-09 holds a gaoled player as retail did (the two conditions and
  untargetable; untested in game: Bob's bundle 20261109-195426 had the player walk out). The gaol is spawned where
  the player stands 4 s after Rock Throw; retail's players were drawn a few yalms (5 in 0.8 s in two throws) before it
  closed, which Kawari doesn't do. Whether enemies leave an untargetable player alone isn't modelled.
* A wipe (Bob's bundle 20261109-195426, the first seen here): the party's NPCs got up in the arena beside the new
  Titan and fought on before the recommencement; fixed (the player's spot at the start kept as they're sent back),
  untested in game. Titan walked about 9 yalms after the bots during the Heart there; retail's walked 0, 5 and 11 in
  the three runs, so that's left as it is.
* ActorControl 30 and 31 on Titan are sent as retail's (`bnpc_control`) without knowing what they do. Since
  2026-11-09 the Heart is spawned as retail's: a part of Titan (BattleNpcSubKind 1, his actor ID in `pet_id`, display
  flags 0x40003), so the client should draw and target it in his chest (`spawn_bnpc_part`; untested in game: as an
  ordinary enemy its target ring stood on the ground before him, Bob's bundle 20261109-201914). It still carries the
  fork's layout ID 99001507 (retail's had none: the director finds it by that) and retail's MP 1080 and target (the
  player) aren't sent. ActionTimeline 1439 is played on Titan, not checked on the Heart.
* The party's NPCs go to retail's Geocrush spots at the arena's edge (tank south, healer north, DPS east; a second DPS
  to the DPS spot, retail's run had one); untested in game. Before, they dodged by the shapes, found nowhere out of it
  and stayed by the player (Bob's bundle 20261109-201914).
* "Dying too easily" (Bob's bundle 20261109-201914, a level 36 conjurer, item level 24, 1035 HP, the party's healer
  with three bots): what Titan dealt matched retail's (Tumult 131-179 against retail's 175, Geocrush 307-532 against
  347-480 at 19-22 yalms, the nearer the more, Earthen Fury 606-655 against 630, Rock Buster 344 on him once against
  357). Both deaths came from Earthen Fury or the Tumult after it with the player at 597 and 721 HP. Retail's level 36
  arcanist had 963 HP, 999 with the **party bonus**, which Kawari doesn't give: about 3 s into a duty retail raised
  Strength, Dexterity, Vitality, Intelligence and Mind by 3 % (one per role in the party: tank, healer, caster; Piety
  unchanged; 109 → 112, 148 → 152 Vitality, capture 650dbfd8). Kawari's player keeps their unbonused stats.
* Weight of the Land and Rock Throw never fell on Titan's target in the runs; Kawari keeps them off it on purpose.
  Retail's helpers cast at a point on the ground; Kawari's are put there and cast on themselves.

## Quests 519 and 520 and the Howling Eye (2026-11-08)

* Untested in game, from three runs (Bob's Chronofoil captures 6a5244ec with Duty Support, 09ecf8e7 and 6c495fc4 by
  the Duty Finder).
* Unseen on retail, so guesses: 519's Elyenora yes/no (scene 9: "yes" to POPRANGE3 in Ul'dah) and 520's (scene 2: to
  POPRANGE0); 520's last zone change taken to be POPRANGE1 (retail's ZoneInit named another pop range, 4107477, but its
  pop range field didn't match the quest's elsewhere either). 519's scene 2 (Lionnellais before the quest) isn't
  played.
* The Howling Eye: Garuda's second Mistral Shriek (64.8 s after the first) and what follows 661 come from the one run
  that got that far; after 661 she only fights by her timeline. Her timeline after she comes back is a fit to the
  runs (656 5.0 s on, 657 11.0, 658 18.8, 659 33.0). The monoliths shelter 2.0 yalms either side of the line (a guess).
  Feather Rain hit no one on retail and does nothing here but its animation. The monoliths never broke in the runs;
  whether they can isn't scripted. Her other statuses (lost on her: 9, 144, 163, 861, 1193) aren't played.

## Quests 963-975 and quest battle 86 (2026-11-08)

* Scripted from three runs each (Bob's Chronofoil captures d7993cf0, 195176f8 and 0d10bf45), untested in game.
* 968, 971 and 974 have no scripts: 3870 took their place and completes them, as retail did; nothing offers them.
* Client scenes never played: 967's 4-7 and 10-13 (empty; the campfires EOBJECT0 and EOBJECT2 are listed for sequence
  3 too, with no scene known for either), 3870's 5 and 6 (an empty scene and the pop-up message, likely for ITEM0
  used on its enemy, which no run did).
* Quest battle 86: territory 330 is taken to be Western La Noscea's ground (the run's spots stand on it). The one who
  talks heals its side below 80 %, every 14 s at most: a fit (retail's came every 14-38 s, at 21-100 %). Its Regen
  ticks only on the one the heal was aimed at. The bombs' loops are retail's positions, not the layout's own path. Bind from the thaumaturges' 1304 lasts 5 s (retail's ended after
  2.5-5.7 s, perhaps when hit, which isn't scripted). The thaumaturges' spells are timed from their pull as a set
  order; retail's nine each used a few in their own order. The fight starts as the lead-in's second scene ends, a
  second before retail's. No lost run was recorded: losing (your health at 0) is sequence 0xFE and out 5 s later, as
  the other battles.

## Quests 953-962 (2026-11-08)

* Scripted from three runs each (Bob's Chronofoil captures 4bcbd425, 18a89a32 and de5beba7), untested in game.
* Unseen on retail, so guesses: 953's Lionnellais yes/no (scene 2: "yes" sends the player to POPRANGE0 in New Gridania);
  962's ferry skipper's yes/no in sequences 2 and 255 (scenes 5 and 8: "yes" takes the player across again); 961's
  lines (4, 7, 10) and 962's (4) by when the client's IsAcceptEvent offers them.
* All Due Precautions (954) is completed with 953, as retail did, with no reward and no script of its own; 957 and 958
  weren't in the runs and have no scripts.

## The Stone Vigil (2026-11-07)

* Untested in game, from three runs (Bob's Chronofoil captures 1cefedf4 with Duty Support, three wipes, 341808b0 a
  tank's and f842581d a conjurer's Duty Finder runs); the bosses' rules are fits to them: Swinge twice below 50 %, the
  last boss's flights below 85 % (at once if its round isn't over), between 50 and 70 % at a round's end and below
  40 %, and a new round as it falls below 50 % mid-round (seen once). Where its
  rounds and flights stand when it's pulled again after a wipe follows the script's reset, not retail's (not seen).
* The first boss's run to its spot is a `sweep_bnpc` at 9 yalms a second (ActorMoves every 500 ms tick, retail's
  every 0.31 s), kept from acting on its own for the run (`busy_bnpc`); it turns to its target as it arrives.
* Hitboxes of the Stone Vigil's enemies aren't recorded (Kawari Recorder wasn't run there); they're their models'
  (`npc_hitbox_radius`), which the cones' and circles' reach start from.
* Where retail's captures had only Bob's tank hit by an action (Chudo-Yudo's and Koshchei's auto-attacks, Rake,
  Spiked Tail, Isgebind's auto-attack, the 1464s' 504, the 1466s' 737), `players` is that times 1.5, the median of
  what the other actions did to the conjurer over what they did to the tank (1.28-2.06); f842581d (the conjurer's
  Duty Finder run) isn't in the release's `chronofoil.zip`, so it wasn't used.
* How often the enemies act is now a rate over the time they fought (`every` = seconds out ÷ uses) from 1cefedf4 and
  341808b0; the first boss's from its three pulls' fighting alone (303 s).
* After a wipe the enemies are put back where they began (an ActorSetPos), where retail spawned the bosses anew; their
  statuses aren't cleared.
* The whirlwinds are untargetable and held (retail's targetability isn't recorded); the helpers that hurt as they go
  cast where the whirlwind is every 0.58 s by the director's tick (500 ms), so some casts come a tick late. Retail's
  kind 62 effect on 28731's hits (173, 64; Kawari's enum calls kind 62 `SummonPet`) isn't sent; whoever has 2656 is
  spared its next casts, as on retail.
* The last boss's Touchdown always comes as it lands; its targets were 0-4 on retail. Its round's Sheet of Ice goes on
  one of the party at random (retail's on whom isn't worked out).
* The perched dragons' ActionTimelines (7749, 7739) are retail's numbers, not checked in game; their breath is cast on
  themselves (a line ahead, as they face in the layout).
* Nothing is sent for the Duty Support members' event NPCs after the clear (none came in either run).
* The Duty Support members' chatter at 41 (the tank's 13954, the Thaumaturge's 13978 and 13979, the Conjurer's 13985)
  isn't added: each member has one list of lines, and whether these replace Brayflox's at 32 by level isn't known.
* Coffer loot: one opening each, and two for 185; the tank's run's first two bosses' items (10438; 31518 and 31527)
  aren't given to a coffer, since which coffer they came from isn't known.
* Party dodging by the duty's spots (`director:party_spots`, `party_positions`) and the sweeps' dangers are untested
  in game; a bot or party NPC whose spot no navmesh path reaches dodges by the shapes. Whether retail's members also
  had spots for the trash, the dragons' breaths and the first boss's Lion's Breath isn't known (they're not set).
* Quest 952's scene 9 (ACTOR5's yes/no) was never seen; "yes" sending the player to POPRANGE1 is a guess. The duty's
  exit spot in New Gridania is both runs' first position there (within 0.4 yalms), the facing taken from the pop range.

## Quest battles and directors

* **Map markers** (2026-11-12, [Quests](quests.md#map-markers)): built from retail's captures to fix the guide's class
  quest staying on one just completed (Bob's bundle 20261112-222427); not seen in game yet, so whether it does is
  open. Which quests retail lists isn't checked against the sheets (Kawari's conditions are the Scenario Guide's plus
  class, level and the kinds left out), nor the icons beyond the startup check; retail's other events in the list
  (handler types like 0xB0147) aren't sent.

* **Notorious Biggs' placement** (2026-11-12, Bob's bundle 20261112-195353): built from four retail runs, untested in
  game ([Quest battles](quest-battles.md#quest-battle-62--notorious-biggs-contentquestbattle62lua)). How far off a big
  enemy everyone stands depends on its hitbox, which comes from the game data's model sizes (`npc_hitbox_radius`)
  when no timeline records one: if the server's log says no formula matched, the 1655s and 1657s count as player-sized
  and are stood 3.5 yalms from, not retail's 5.5. Other battles' waves now stand 4.9 s too (`ARRIVAL_STAND_SECONDS`),
  from Biggs', QB 61's and To Guard a Guardian's retail runs; a battle whose retail adds came at once would need
  `stands_for = 0`.
* **Party bots' gear below level 50 is capped at their level** (2026-11-10), untested in game: whether the HP that
  gives matches retail's Duty Finder players at that level (about 1500 at 40-41 in capture cc62ad94) wasn't checked
  against the game data here.
* **Mounted players and events** (2026-11-10): an ordinary event no longer sends SetMode, and enemies wait while their
  target is mounted ([Quests](quests.md#an-event-leaves-the-players-mode-alone)); untested in game. Something that
  relied on the old SetMode Normal at an event's end (a mode left set by a script) would now keep it.

* **In the Eyes of Gods and Men's battle (60)** (2026-11-06): untested in game, from three won runs (captures 130088ec,
  3c56e09f, 21f967d2). Picking up the rosary is new: the director answers a talk on its object with an event action and
  no scene (`onObjectTalk`), which the client hasn't been seen to accept from Kawari yet. Whether sequence 6 waits for
  sequence 5 isn't known (the boss reached 50 % after it in all three runs, once 7.4 s after). The boss's own Cure
  under 15 % fits all three runs only roughly (its first came 0.7-8 s after it crossed). The wyvern is invulnerable and leaves on its own clock; on retail it stayed on the player (it gains no enmity
  from hits since 2026-11-10), but whether retail's ignores Provoke too isn't known. 1738's one Provoke (18) on the
  boss is scripted at 2.5 s after he arrives (retail 2.0-9.6 s), not from a rule; 1739's Stoneskin (129) isn't in its
  timeline. Since 2026-11-10's targets (bundle 20261110-095652, "far more difficult" than retail) untested in game. Losing as
  Haurchefant falls is from Bob's memory of retail, not a recording: whether the conjurer ally's fall loses it too,
  and what retail says or shows then, isn't known. Haurchefant's ride in (untested in game) follows straight lines
  between retail's points at their heights (the ground rises 2 yalms on the second leg; the client may show him off
  it). The wyvern's run off as the boss falls (2026-11-10) is a straight line too, untested in game.
  Bob's first play (bundle 20261110-081839, a level 38 conjurer) stalled on the first wave's conjurer (915 HP): it
  Cured itself for 435-475 every 6-7 s and Cure II'd for 712-737 under 24 %, retail's amounts (one Cure took it from
  2 % to 53 %) and pace (6.3 s median between its heals, 6.6 s for this one). Retail's players hit it for 210-300 a
  spell and it died between two Cures; Bob's Stone II did 130-189, and your conjurer attacked every 13.5 s where
  retail's did every 7.6 s (fixed). Whether Kawari's spell damage at level 38-40 is retail's isn't checked (only levels
  6-9 were: [Combat](combat.md)); the three runs' players' classes aren't in the fixtures.
* **The Heretic among Us's battle (61)** (2026-11-06): untested in game, from five runs (three won, two lost). Dragon's
  Blood is the director's, 12.0 s after sequence 4 (retail's 11.1-13.0 in five runs); whether something else timed it
  isn't known. The boss's adds of sequence 3 needn't be down for sequence 4 (one was still up in the third character's
  run, 21f967d2).
  The allies' Stoneskin (129) is left out; Bio's damage over time isn't scripted (only its status), nor what Eye for an
  Eye and Rouse do (their statuses, 20 s, a guess). The adds' routes down (2026-11-10) are untested in game: they're
  straight lines (`sweep_bnpc`), so the client may show them walking through the air on the drop, and where they land
  has to be on the arena's navmesh for them to fight on. 4256797 fights from its ledge with a 15-yalm reach; if its
  target moves further it walks off towards it and into the cliff as before. The archer ally's 18190 (six uses in
  five runs) isn't in its timeline.
* **Timelines whose `every` is the median gap between uses** (2026-11-10): QB60's and QB61's are now rates (uses over
  the seconds out, [Quest battles](quest-battles.md#quest-battle-61--the-heretic-among-us-contentquestbattle61lua)),
  but 41 more written the same way overstate how often their NPCs act (most by 1.2-2.5 times: QB60's marauder used
  Overpower every 11.6 s against retail's 26.5): ByTheLightsOfIshgard_46, ClsGla250Enemy_1243, ClsGla300*, EyesOnMe_1902,
  FactualFolklore_2196, QB20*, QB21*, QB23*, RoadToRedemption_1904, StoneVigilBoss1_1461, StoneVigilTrash_1462-1466,
  TheUnendingWar_724 and 725, ThreeForThree_726. Their captures are needed to redo them.
* **Field enemies' timelines of 2026-11-06** (724, 725, 726, 1902, 1904, 2196, 46): from the quests' fights alone (two
  runs each, four for 2196); base 46 and 724-726 live elsewhere in the world too, where these timelines now apply.

* **The Lominsan Way's battle (59)** (2026-11-03): untested in game, from seven runs (five lost, two won; captures
  44df02fc and c9145fdc). Its lead-in (scene 2 played from `onCommence`, the fight from its end) is new: the client
  hasn't been seen to accept a GameProgress scene right after the opening this way. Paralyze comes 3.0 s after
  sequence 4 (retail's 1.7-3.6) and sequence 5 5.0 s after it began, by the director's clock, not the cast's
  (interrupted, the sequence still comes). 1769's 15 s line and Heavy Swings come from its timeline on a fixed clock;
  retail's second line was 15 or 32 s after the first. What happens if the bomb's blast misses 1771 (it was always
  beside it) or the player kills 1769 before the fight starts isn't known. The onlooker's spawn uses `spawn_bnpc` on an
  event NPC, as Brotherly Love's does.
* **Quest battle 17 (quest 441)** (2026-11-01): from three runs (two lost, one won; capture df6f5af9; quest 441
  scripted from Bob's dump the same day). Bob played the quest and won the battle on 2026-11-02; its reward then
  crashed the server (an action's UnlockLink taken as an unlock bit, fixed the same day): the reward is untested
  since. Whether the fight
  really waits for every add once the boss is down (Bob's reading) wasn't seen: in the won run they were all down first.
  The boss's drain is in its rotation, so when it comes depends on what else it's doing (retail's first came 1.6 s
  after the first yodomi in one run and about 10 s after in the others). The paralysis's 16 s is from when it was lost
  where nothing cut it short.
* **Big Trouble in Little Ala Mhigo's battle (58)** (2026-10-31): untested in game, from three won runs (captures
  20ef0e90, 9faa9bac, bd51ca23; its client battle script has only the opening and closing scenes and sequences 1-2's
  objective). Sequence 5's three ways in (the second wave down, the boss at 40 %, 50 s after 4) were each seen once;
  the boss's mark is between 40.9 and 39.7 %. What happens if Gundobald or Wilred falls (the conjurer fell in one run
  and nothing changed), or if you don't fight, isn't known. Retail had the first wave and your side out from the moment you arrived;
  Kawari brings them out as the fight starts. The third wave's pugilists' self-heal (57) is a fixed 52 (retail's
  338); the statuses of 426 (204), 632 (197), 1288 (15), the boss's Thunder II (162) and the conjurer's Aero (143)
  aren't scripted beyond being applied, and their durations are from when they were lost.
* **Nouveau Riche's battle (13)** (2026-10-30): untested in game, from three won runs. Sequences 4 and 5 come 30 s apart
  as on retail; whether either waits for the wave before it isn't known (in every run it was down in time). Sequence 3
  came 3.0, 1.1 and 2.0 s after the last lancer's health reached 0: Kawari answers at 2 s. The second wave comes back at sequence 5 only
  where it has fallen.
* **Brotherly Love's battle (57)** (2026-10-30): untested in game, from three won runs (its client battle script, dumped
  2026-10-30, has only the opening and closing scenes and sequences 1-2's objective). Sequence 3 at the first
  thaumaturge's death or 30 s, sequence 4 at 37 s and sequence 6 at the leader's 40 % are readings of the runs
  (sequence 6 came 26.2, 2.6 and 0.1 s after sequence 5, at 35 and 39.9 % and with the leader already down, so not a
  timer). Sequence 4 came at 36.5, 37.0 and 38.6 s: what holds it past 37 isn't known (no death, cast or position fits
  all three). What happens if you don't fight isn't known. The beast's 604 is a hit to 20 % of Yda's and
  Papalymo's HP, cast once by the director: retail's effect on them was kind 48 (unknown to Kawari; a knockback? they
  moved 26-37 yalms over the fight), the first left both at exactly 80 % and the second took nothing. Yda's 63 and
  Papalymo's 151 durations are guesses, and the statuses they give aren't scripted. Played by Bob (bundles
  20261030-110117 and -110334): Yda stopped short of the beast for good (a wall; NPCs in content now walk on through
  one after 2 s, not yet tested in game) and the beast vanished at the end instead of running off and fading (it runs
  home and fades now, at Kawari's run speed, not retail's slower one; NPCs have no speed of their own yet). Where the
  wall was isn't known: the server logs the next one (`held up by a wall`). Open-world enemies walk on too since
  2026-11-01 (a Ziz held up for good in the North Shroud, Bob's bundle 20261101-215226): the collision there stops
  steps the navmesh allows, and where isn't known yet, the log says (`held up by a wall at`).
* **AoE amounts from Chronofoil runs before 2026-10-30**: questkit read AoeEffects' effects six bytes off (fixed), so
  timelines' `recorded_amounts` for area actions taken from those runs' reports (Like Mother, Like Daughter's 1684s'
  440, others) should be read again from their runs.
* **The Company You Keep's battles (46, 47, 48)**: untested in game (2026-10-28). Sequences 5-8 (the vanguard at 70
  and 40 %, then its Drill Cannons) fit the three won runs, and nothing else was seen to come with them; that the
  Drill Cannons, not something they come with, move it on is a reading of the runs (2.85-2.9 s after the cast began
  every time). Only 46's conjurer's last words are recorded (none for 47's or 48's allies, nor a loss of either: they
  fail as 46 does). No client battle script was dumped for any of them. Flash, Rampart, Raging Strikes, Life Surge
  and 983 aren't modelled, nor 48's archer's and conjurer's own actions beyond their timelines. Battle 48 is from one
  run, 681's quest (also one run) untested in game.
* **The companies' first steps** (683-685, Grand Company enlistment, 2026-10-28): untested in game (684 replays
  against an arcanist's run, Bob's capture cc62ad94). An achievement's ActorControl 518 (2026-10-28, after 515, to
  the player and those around) is sent as retail did; what the client does with it isn't known. A company's Hunting Log
  stays locked after enlisting (retail sent nothing for it then; when it opens isn't recorded), and the seals aren't
  capped by rank.

* **NPC shouts** (`Timeline::shouts`, combat.md): the lines are retail's for the battles in Bob's captures, from one to
  seven runs each. Where a line came with a later use (Garibald's fifth Heavy Swing, Yda's (249) fifth 70, Miraudont's
  third 82) it's that use's, from one run; an HP or time trigger would fit as well, and isn't known. What the client
  shows for a line (a balloon, chat) isn't checked in game.

* **Into a Copper Hell (44)**: untested in game (2026-10-25). Territory 274's drop-in (`wil_w1t1`) appends a guessed LGB
  for Ul'dah - Steps of Nald; if 274 loads other LGBs, the three adventurers won't spawn. Statuses 71, 99 are only applied.

* **Battle talk**: every quest battle uses the window (DirectorPopupMessage) with retail's text IDs, except Papalymo's
  line when he heals you in Chasing Shadows, still said in `/say`: retail's text ID for it hasn't been seen (he didn't
  heal in the capture). Its neighbours suggest 129, unconfirmed.
* After winning a quest battle, Kawari moves the quest on as "Duty Complete" plays; retail sends the UpdateQuest after
  the closing scene (seen in all four battles of Bob's Chronofoil captures). The client doesn't seem to mind.
* Open-world monsters' speech bubbles aren't sent (retail: an idle level 6 monster in the Central Shroud, and one
  fighting other NPCs, each with a value in NpcYell's `param1`, 88 and 37, whose meaning isn't known).
* Chasing Shadows' enemies only auto-attack (no skills yet).
* NPCs fighting each other without `recorded_amounts` use their stat-based damage (unscaled); only Leia's Legacy and
  To Guard a Guardian have recorded amounts so far.
* An NPC's area attack aimed at another NPC only hits that NPC (the player standing in it isn't caught); on retail the
  lostwings' Power Wave sometimes caught the player too.
* The 24 action scripts added for these battles' NPCs (Heartstopper, Quick Nock, Fire II…) have their damage from the
  recording for those NPCs; their potency for a player who uses the same action isn't verified, and none of them give
  their statuses (Windbite, Venomous Bite, Terror Eye…).
* When allies heal (thresholds and how often) is fitted to the recording, not known rules.
* How near Papalymo stays to you in Chasing Shadows (15 yalms) is a guess until a recording of it shows it.
  `keep_near` takes the NPC to its anchor itself (not to the edge of the distance), and retargets don't consider
  whether a target is out of the anchor's reach.
* Class rotations time a re-applied debuff per target (`refresh`) instead of reading the target's statuses, and
  don't yet know procs (`ActionProcStatus`) or charges. The quest battles' timelines are still fixed timepoints:
  they move to rotations as Recorder 2.7 recordings of them come in.
* Eastern, Southern and Northern Thanalan's and Mor Dhona's monsters have no timelines (no Recorder export of those
  zones): default auto-attack, hitbox and damage, no skills. Northern Thanalan's drop-in only has the 99 spawn points
  of a short visit.
* Kawari sends character data flags 0 for passive monsters, where retail sends 0x1, and no Behavior for monsters
  (retail: 30246 for most); what either changes in the client isn't known.
* Guards ([Combat](combat.md)): untested in game since they fight. They notice monsters with the aggressive monsters'
  rule (15 yalms in front, 11 all round), measured from one retail engagement; none of their kinds has a timeline, so
  they auto-attack only (retail's Lalafell also used 717 and 718). Talking to one that's fighting plays its line
  (retail ended the event at once).
* `stationary` in the Thanalan and Mor Dhona drop-ins came from a one-off scan; `questkit chronofoil --dropin` writes it
  now (La Noscea's). Regenerating Western Thanalan with it agrees on 382 of 397 spots: the 15 others (bases 12, 42,
  131, 187, 313, 431, 769, 1445) are marked stationary but were seen wandering while idle, several times each, in the
  captures since; not changed yet.
* La Noscea's monsters have no timelines (default auto-attack, hitbox and damage), and its zones' `Bg` folders weren't
  checked against the game (see Combat).
* To Guard a Guardian: which six Ixali come first is a guess from where they stand. Where the navmesh has holes,
  NPCs in content walk straight across them (rocks included) rather than stall the battle. Dying sends you back out to the
  instructor (as any lost quest battle does); whether retail offers a retry first isn't recorded.
* The new quest battles' timelines (which skills, how often) are loops fitted to how many times the recording saw
  each skill used.
* Retail sends the player out of a lost battle; the delays are copied from recordings, not from known rules.
* Lord of the Inferno's battle (45): won in game (Bob's bundle of 2026-10-28); the exit to Southern Thanalan after it
  is untested. Territory 275's own layer `QB_ManFst209_001`
  (86868, filter 86631) has event NPCs 1004667 and 1004668; retail showed only 1004668 (the scout, which the script
  also spawns from the drop-in), so the scout may show twice and 1004667 once.
* Lord of the Inferno's battle (45): the 55 s from the third wave to the win held in all three won runs (55.1 s each);
  the second wave's 30 s from four runs. Retail's NPCs fought on through the closing
  cutscene, Kawari's stop, so the second gladiator's last words after the win (retail's first won run) won't come. The Flames' 14 and 718
  and the Amalj'aa's 426 aren't modelled.
* Underneath the Sultantree: the stranger's Flash, Sentinel and Sword Oath (seen on retail, none of them damage) have
  no action scripts and aren't in his rotation. The third pair of lesser blangas comes at 23 % of the blanga's health;
  in the one run it came between 24.2 % and 22.1 %, also 15 s after the second pair, so it might be timed. The blanga
  never went after the player on retail, so its hits on a player use the generic damage. Kawari ends the battle once
  everything is down (Sapphire's rule); on retail the blanga happened to fall last.

## Walking on the collision (2026-10-26)

* Untested in game: it was checked on made-up geometry (slopes, steps, walls, a ledge, a bridge) only, since the
  workspace has no game files. The first entry into each zone on this build makes its navmesh and collision again
  (`kawari-navimesh`, in the background; open-world zones take a while), and until then NPCs keep to the navmesh's
  floor as before.
* Since 2026-11-10 (64-bit polygon references) every zone's navmesh is made again on its first entry, and until it
  is the zone has none: enemies and bots there walk straight at their targets over the collision for that minute or
  so. The fix for Coerthas Central Highlands (the world server panicked loading its navmesh, Bob's log of 2026-11-10)
  was checked by tests only (a navmesh of 5000 tiles of up to 600 polygons is taken; an older file isn't read; one
  Detour won't take is left out without a panic): the workspace has no game files, so the zone's own numbers aren't
  known.
* The collision is all of a zone's collision meshes as they load, as the navmesh's is: a collision box that a
  layer switches off later (or on) is still there (or not), for NPCs as for the navmesh. Bg parts whose collision
  isn't a "replace" mesh (`ModelCollisionType::Replace`) and sphere, cylinder and plane collision boxes aren't in
  either.
* Gimmicks (an event object's bound shared groups) go from the collision when the server hides the object, not
  when a shared group's own timeline plays (a door swinging open on retail while its object stays): such a door's
  collision stays shut. An event object the server doesn't spawn (pop type other than 1) leaves its gimmick standing.
  Untested in game: whether the Copperbell Mines' entrance barrier and Tam-Tara Deepcroft's hold-ups were gimmicks is
  what the next runs' logs say (the navmesh tool's count, `The gimmick of event object …` and `held up by the gimmick`
  lines, or `held up by the zone's collision, and no ground` when it was something else).
* The navmesh tool builds from every layer of the zone's LGBs, the world server spawns from those in the territory's
  layer set only; the tool now logs how many collision triangles come from the others, to see whether that matters.
* Bots don't ride lifts: they're put beside the player once 25 yalms behind for 3 s (`CATCH_UP_DISTANCE`,
  `CATCH_UP_AFTER`, guesses), not carried down with them. Retail's Duty Support on a lift isn't recorded.
* An NPC already inside a wall or a rock (left there by earlier straight walking, or spawned in one) can be boxed in:
  every way out goes into the far side of it.
* A ledge's drop is instant (it's on the floor below the next tick, and the client glides it down over the half
  second), not a fall at a player's speed.
* `OPEN_WORLD_MAX_DROP` and `PATH_DROP_MARGIN` (3 yalms) are guesses, and `AREA_HALF_HEIGHT` (5) too; and how retail's Duty Support members follow a player who jumps off a
  ledge (down after them, or round by the path) isn't recorded.
* The generator's other changes (the heights the navmesh covers taken from the geometry, the climb in cell heights,
  ledges filtered) change every navmesh made again; paths that worked could differ.

## Combat

* Passive monsters (no HOSTILE character flag) are on the monsters' side again, by battalion (2026-11-04, untested in
  game): a player's single-target hits on one give enmity, so it fights back and its kill is credited (quests,
  Hunting Log, EXP). Kills of passive monsters from 2026-11-01 to then weren't counted anywhere and aren't made up.
* ClientTrigger 441 is unknown: it came twice just before an EquipGearset (Lira Ormont, 2026-10-26); retail's meaning wasn't checked.
* NPCs no longer redraw their weapon after every action (2026-10-25: retail never did; the redraw dated from the
  old 0.6 s animation lock and swing 0, which now match retail). If NPCs drop their battle stance between attacks
  again, look at the NPC ActionEffect.
* Area results beyond eight targets fall back to flying text (only AoeEffect8 is laid out, not 16/24/32).
* **Duty Finder bots** (2026-10-22, untested in game): bots are listed as a Duty Finder party of players (retail's
  Duty Support list, recorded on 2026-10-23, is for [Party NPCs](party-npcs.md)); whether the client takes an NPC's actor ID there (HP bars, targeting
  from the list) is the first thing to check. Their HP in the list isn't updated by the server, no PartyMemberPositions
  are sent for them, and they don't know Sastasha's route (the tank bot follows you).
* **Carbuncle** (2026-10-22, untested in game): it isn't summoned again after zoning (retail resummons it as you load
  in), summoning again doesn't dismiss the one out, it doesn't fight, and pet actions other than Away (Heel, Place…)
  do nothing. The 50-yalm catch-up distance is a guess.
* **Enemies and the navmesh's edge**: an enemy walks straight on from the edge to a player within 15 yalms of it; a
  player standing off the navmesh further than that can't be reached, and after 7 s the enemy gives up if the player
  is beyond its leash (since 2026-10-22; before, it gave up at once).

* **Enemy damage** without a `damage_to_players` recording falls back to 12 % of a player-style calculation; retail's
  enemy formula is unknown. Only Central Shroud monsters have recordings.
* **Area attacks** on several players show only the main target in the ActionEffect packet (only the one-target
  opcode is known); the others get flying text and HP updates.
* **Drop rates** are a 50 % guess except for anoles, ground squirrels and trickster imps; drops cover the Central
  Shroud only.
* **Aggro** (11 yalms all round, 15 in front), **leash distance** (40 yalms) and speeds (6 and 2 yalms a second) are
  global; retail's assist (same-kind enemies joining) isn't done. Respawn delays follow retail's level steps but not
  its 120 s spots. Regeneration ticks once per instance rather than on each actor's own phase. Enemies above level 50
  outside Heavensward and Shadowbringers field zones, and duty enemies without an `hp`, still use the player-style
  HP.
* Player **HP** ([Combat](combat.md)) is retail's at every measured level; levels 23–29, 38–49, 55–59, 61–66,
  71–75, 77–79 and 91–100 are interpolated (or extrapolated), and tanks below 50 and above 80 use a ratio. The party
  bonus (+1–5 % on retail, a few seconds after zoning) and food's HP aren't applied.
* **Class actions 1–50** ([Combat](combat.md#class-and-role-actions-levels-150)): potencies are 7.x max-level tooltip
  values, not checked at low level (only Conjurer/White Mage are in Bob's captures). No egis or fairy, no
  positionals, movement actions do nothing on the server, barriers don't absorb damage, Bloodbath doesn't heal, Cover
  doesn't redirect, Thrill of Battle doesn't raise max HP, Peloton is the caster's only.
* **The ARR jobs at 51–100** ([Combat](combat.md#the-arr-jobs-levels-51100), 2026-11-10, untested in game): the
  potencies are the job guide's level-100 values at every level; the ActorGauge offsets of the Warrior, Monk, Dragoon,
  Bard, Ninja, Black Mage and Scholar gauges are FFXIVClientStructs' less 7 as the Paladin's and Summoner's are in Bob's
  captures, not seen for those jobs (a gauge showing nothing or the wrong thing would be that); ground-targeted actions
  (Liturgy of the Bell, Sacred Soil, Ley Lines, Doton, Earthly Star) put their effect on the caster or the target;
  Ley Lines' and the Army's Paeon's speed-ups, Triplecast, the Mage's Ballad's Bloodletter resets, Ten Chi Jin's
  three-mudra lock, Deployment Tactics' spread, Excogitation's trigger (it heals at once), Fey Union's drain, Expedient's
  speed and the demi-summons and Seraph as pets aren't done; Lightwyrm Trance uses Dreadwyrm Trance's status (no row of
  its own in Boss Mod's list). The Summoner's arcanum order and attunement counts and the Ninja's mudra IDs 18805-18807
  are from retail's captures; the rest of the mechanics are from the job guide's text.
* **The jobs from Heavensward on** ([Combat](combat.md#the-jobs-from-heavensward-on-levels-1100), 2026-11-10,
  untested in game): potencies are the job guide's level-100 numbers, with the lower ones its traits list below their
  levels; under the lowest listed level the lowest listed stands (Dark Knight's, Gunbreaker's and Machinist's below 84,
  for instance: retail's are lower still). Their gauges' offsets are FFXIVClientStructs' less 7 (only the Dark Knight's
  is seen in Bob's captures). No pets (the simulacrum, turret and queen hit with the job's weaponskills; the Pictomancer's
  muses hit at once), no positionals, barriers don't absorb (so The Blackest Night's Dark Arts comes at once, and the
  Sage gets no Addersting: Toxikon is never usable), Kardia's heals, Synastry's echo, Collective Unconscious's regen,
  Excogitation-like delayed heals (Exaltation, Catharsis of Corundum, Vigilant) land at once or not at all, haste
  (Fuka, Swiftscaled) isn't counted, ground areas (Salted Earth, Earthly Star, Doton) go on the target or the caster,
  movement actions (Shadowstride, Trajectory, Slither, En Avant, Smudge, Hell's Ingress, the Red Mage's Corps-a-corps,
  Displacement) don't move anyone, Devilment's and Chain Stratagem's critical rates aren't counted, Heat Blast doesn't
  take time off Gauss Round and Ricochet, the Scarlet Delirium chain's MP (200) is a guess, and the Red Mage's 45960-45962
  are taken to be the Magicked Swordplay Enchanted ones. Bots of these jobs (now in the Duty Finder draw) are untested
  in game: their gear, glamours and weapons by ClassJobCategory.
* **Healer bots' heals** are one single-target GCD heal (`heal_action`); they don't use the healers' abilities
  (Lustrate, Excogitation, Indomitability, Tetragrammaton, Divine Benison...) or area heals, and the rotation scripts
  can't see the party's HP.
* **Combat items:** HQ items are used (since 2026-11-12) but give their NQ values, but for food's (Well Fed's HQ
  param); elixirs give their caps.
* **Item recasts**: retail's ActorControlSelf 17 for an action (sent by Kawari since 2026-10-18, `action.rs`; 72 of
  77 action requests in Bob's bundle 20261111-194208, the rest mounts) comes for an item too when it has a recast, with
  its own group, an action of its own and the Item row's recast: item 8575 `(60, 2894, 180000)`, 20309
  `(59, 8667, 1500)`, 25005 `(58, 944, 600)`, a HQ potion 1006141 `(68, 1055, 21600)` (Bob's Chronofoil captures,
  2026-11-11 survey); most items had none. Where the group (58-68) comes from isn't in the Item or ItemAction rows
  icarus reads, and there's no game install here to find out, so Kawari sends none for items and doesn't track their
  recasts (the client keeps its own).
* Barriers (since 2026-11-11, [Combat](combat.md#barriers)) are the 7.x tooltips' sizes; retail's Galvanize after
  Adloquium left 2.43-2.50 times the heal as shield nine times in Bob's captures (other players' Scholars, Duty Finder)
  and 1.18-1.93 four times, not worked out (a critical heal's Catalyze, two barriers at once, or the tooltip being
  another level's). Celestial Intersection and Improvised Finish give no barrier yet (their scripts give no status to
  hold one); Haima's five Haimatinon don't renew it; Divine Caress doesn't leave Divine Aura.
* Sprint and Jog over a zone change: Kawari drops statuses there; retail sometimes kept Jog in the new zone's list and
  sometimes didn't (no rule found yet, [Combat](combat.md#sprint-and-jog)). 8 of 37 retail Sprints lasted 10 s instead
  of 20 (all on the new 2026.10.06 characters), unexplained.
* No Limit Break gauge or Limit Break actions.

* **Teleport animation stayed on** after a teleport within the same zone (Bentbranch, 2026-10-16; Moraby Drydocks,
  2026-10-22): Kawari sent warp type 1 where retail sends 4 (see [Travel](travel.md#teleporting)); fixed 2026-10-22,
  untested in game. Retail's ActorControlSelf 263/253 around the warp still aren't sent.
* Anything that keeps a player's connection busy for a second or so (a slow sheet read in a loop, say) desyncs the
  client's Oodle stream: every later packet from it fails to parse and the player is stuck (frozen, endless loading)
  until they reconnect. Why a delay breaks the stream, rather than only slowing it, isn't known.

## Items

* Items that teach **folklore** (4107) or **secret recipe books** (2136) aren't handled (the server doesn't keep those
  unlocks yet); they say "Unhandled item type".
* Learning something from an item has no log message of its own on Kawari; whether retail sends one (or the client
  prints it from the actor control) isn't known.

## Spiritbond, condition and materia extraction

* Materia extraction and self-repair follow retail's desynthesis flow (ClientTrigger 2800, EventStart type 0x19, scene
  0, yield, return); neither was captured, so their result packets aren't sent and the flow is untested in game.
* Combat spiritbond at duty/FATE completion is a stand-in fitted to the captures (`spiritbond::content_base`).
* Quick synthesis doesn't wear gear per item; failed duties and FATEs don't wear or bond at all (retail: treasure
  dungeons and FATEs still do); fishing, Reduced Durability Loss and the self-repair "repair all" mode 3 aren't done.
* Gear for every class (MaterializeType 1, 7, 10) extracts materia of the player's current class's role (a guess).

## Character windows

* **Hair highlights** of characters saved before `77d5315`, or imported from an Auracite dump (its plugin always
  writes `has_highlights` false), are off until set again at the aesthetician (or with a Fantasia) (character windows, "Hair highlights").

* **Achievements**: retail follows every `AchievementCompleted` (515) with ActorControl 518 and the same ID; Kawari
  doesn't send 518 (it isn't in `ActorControlCategory`), so what others nearby see when you earn one is probably missing.

* **Adventurer Plate**: Kawari has no free companies, so the crest and FC name are empty. CharaCardData's byte 0x43 (`unk13`, 1 on retail), the state number at 0x1C (2) and Data32's first u32 (0 on
  retail; FFXIVClientStructs calls it a timestamp) aren't understood. Plates saved before
  `update-2026-10-20_adventurer_plate_saved` show timestamp 0 until saved again.
* **Achievements**: only Types 2, 3, 6, 8, 9 and 20 and counter 11 are completed automatically (of 33 Types and 1128
  counters), and what those Types mean is the community's reading of the sheet, not checked on retail. Mapping the
  Realm needs every part of the map discovered, and Kawari reveals map parts one at a time as the player walks into
  them (retail sometimes reveals a whole map at once, see `server/zone.rs`), so it depends on that working for every
  part of a zone. Nearly completed lists are always empty. Whether retail also sends
  LogMessage 952 ("… earned the achievement …") on completion, or the client prints it, isn't known.

* `!` debug commands are only recognised in zone chat (Say/Shout/Yell); in Party, Tell, Linkshell, Free Company or
  Novice Network chat they're posted as ordinary messages (upstream's behaviour).

## Glamour dresser, plates and armoire

* New and untested in game (2026-10-24): the dresser, plates and armoire follow Bob's retail capture.
* Applying a glamour plate (character window, gearsets with a plate linked) and the mender's repairs follow Bob's
  capture of 2026-10-24 and are untested in game. ActorControlSelf 1801 and the repair ones (90, 91, 93) are sent as
  retail without being understood; 57 and 804 after a plate aren't sent.
* A gearset's linked plate (2026-11-02): the plain EquipGearset's plate byte is now applied, as retail did (until now
  only EquipGearsetGlamour's was); untested in game. Bob's client sent no plate at all for his linked gearset in Old
  Gridania (three clicks at two places, each soon after he had set a portrait) and did in New Gridania: why isn't
  known (the client's own check; Kawari has nothing to apply then).
* Not done: outfit sets, armoire items on plates, glamour prisms, dyeing a dresser item, self-repair, materia
  extraction. SaveGlamourPlate's last 124 bytes and its second 12 flags aren't understood; ArmoireInfo's meaning
  isn't known (it's sent as retail's 1 and zeros).
* Gold Saucer tabs: the Chocobo tab (1900) isn't answered (opcode 661 is the racing chocobo, RaceChocoboManager's
  layout, and Kawari has none; ActorControlSelf 933 `[3]` isn't understood), nor is 2010 (opcode 410, AgentId 0xB9,
  layout partly read); GoldSaucerInformation for the General tab is sent as zeros. Mahjong's 2001 `[0, 1500, 1500]`
  is sent. Nor is ClientTrigger 1011 (`[achievement]`, answered with ActorControlSelf 600
  `[achievement, 0, 0, 1]`), the free company's activity list (client opcode 708, paged), or CustomTalk 200
  (CmnDefMiniGame) at the Hourglass.

## Crafting

* Rewritten to Bob's retail capture of 2026-10-24 and untested in game since. Retail's pauses (the GainEffects ~2 s
  after a step, back to the log 1.5–3 s after the item, a quick synthesis' items every ~2 s) are sent at once.
* Scene params 1–3 and 18 were 0 on retail too; StartCraft's 136153 isn't understood.
* EXP is a stand-in fitted to three retail numbers; the first-time bonus (retail's second message, +350%) isn't given.
* Condition chances are Craftimizer's (Teamcraft's differ); whether Heart and Soul, Quick Innovation and Careful
  Observation change the condition is disputed (Kawari: only Careful Observation does).
* The buffs' Action rows also come as ActionRequests, which the action pipeline sees as unscripted actions (a warning).
* An emptied material stack goes as an UpdateInventorySlot of an empty item (as item moves do); retail's capture had
  no stack emptied by a synthesis.
* Recipes made before 2026-10-26 aren't in the Crafting Log (Quick Synthesis) until made again.
* Not done: trial synthesis, collectables (collectability, CollectablesShop), food and potions, specialist and Manipulation's quest unlock
  (the client greys those actions; the server doesn't check), crafting leves.

## Gathering

* **Fishing** (2026-11-13): what bites, when and how hard is built from the community's catch logs
  (`fishing.rs`, [Gathering](gathering.md#fishing)); the packets aren't, for want of a retail capture of some fishing.
  Lures, Patience, Chum, Identical Cast, Surface Slap, ocean fishing and spearfishing aren't in the rules.
* **Triple Triad** (2026-11-13): the match's rules are built (`triple_triad.rs`, [Gold Saucer](gold-saucer.md#triple-triad));
  the packets, the NPCs' decks read from the TripleTriad sheet, retail's NPC choice of move and the rewards aren't,
  for want of a retail capture of a match.

* Rewritten to Bob's retail capture of 2026-10-24 and untested in game since. Actions at a node go through the action
  pipeline as if the client had sent an ActionRequest; whether the pipeline lets them through while at a node isn't
  tested.
* The window's slots keep the sheet's order (retail shuffles them per base); the "rare" flag (16) and the boon 255 of
  some unspoiled items aren't worked out; a level 60 harvesting node once had flags 0x798.
* The gathering chance is fitted to two old (2013) measurements (retail showed 100 everywhere for Bob's high-level
  gatherers); a capture of a low-level gatherer on items near its level would check it; EXP below 22 levels under the gatherer and the
  ranged node bonuses' upper ends aren't known; one returning point in the capture (ActorControlSelf 109 for 30631
  where 30503 was expected) isn't explained.
* GP is gear only (no base GP known; ParamGrow has none); the GP traits' quests aren't checked.
* Points come from Bob's captures: 20 zones only (none in Western La Noscea, the Sea of Clouds and most of
  Thanalan, nor Endwalker's and Dawntrail's zones). GatherBuddy Reborn's `world_locations.json` has 2,116 points'
  positions but no layout IDs.
* The Giving Land gives its status but no extra crystals (its yield isn't known); Revisit, Nymeia's Ward, Eureka
  Moment's traits, collectables (Collect, Scour, Brazen, Meticulous, Scrutiny, Collector's Focus, Priming Touch),
  folklore tomes, Truth being needed for unspoiled points, Lay of the Land, hidden items' chances and treasure maps
  aren't done. Spearfishing and fishing are separate (upstream's fishing).
* A used-up timed point stays visible (empty) until it despawns; each player sees their own phases and integrity.

## Grand Companies

See [Grand Companies](grand-company.md). Untested in game (2026-10-29).

* **Promotions** follow the client's script; retail's packets for one aren't recorded (no capture reached a
  promotion), so the order (seals, rank, GrandCompanyInfo, then the ceremony) is a guess, and what follows a rank's
  reward scene (5, 6, 9, 10) too. What the new rank unlocks isn't given: expert delivery, the company leves, changing
  companies.
* **Changing companies** (the officer's scene 7, for a member of another company) isn't done; the event ends.
* **Supply and provisioning missions** open and close, but handing items in (seals, EXP) isn't recorded or done.
* **The quartermaster** sends no InventoryTransactions (retail sent three a purchase). Ventures (21072, which Kawari
  keeps as a currency) bought there aren't checked. A purchase over the seals the client showed is refused with no
  message.
* **Seals** still aren't capped by rank (GrandCompanyRank's MaxSeals: 10,000 at rank 1).
* **My Little Chocobo**: a name longer than 17 bytes comes in a text yield of a size not recorded (the client's
  limit isn't known: Kawari takes up to 20 characters), and retail's answer to an unusable name isn't known (Kawari
  resumes the yield with 0). Retail's Unknown452 after the hand-over and Unknown159 (seen after the officer's menu and
  the whistle) aren't sent. The Maelstrom's version (701) has only its acceptance recorded.

## Chocobo companion

* Untested in game (2026-10-24). Its fighting is from one capture at level 88 with Attacker at 6: how its HP and
  damage grow with level (ParamGrow's HpModifier is the stand-in), its skills' timing in Defender Stance, and the
  whole of Healer Stance (the Wiki's account) aren't recorded. Choco Guard does nothing (its status isn't known),
  Choco Surge doesn't raise healing, Choco Regen's ticks aren't scaled, Choco Medica heals one target, the
  Accuracy, Critical Hit and Attack Speed traits do nothing, and its attack range (3 yalms) is a guess.
* Companion EXP from a kill is a stand-in: as much as its owner's base EXP for it (retail's monsters well below Bob
  gave neither of them any). Retail's rank-up and onion packets aren't recorded (only CompanionUpdate is sent); the
  Challenge Log's Bosom Buddies, chocobo raising at a stable, the favourite feed, colouring, renaming (My Little Chocobo names it), its party-list
  entry, Examine on it and pillion on one's own chocobo aren't done.
* ActorControl 36/267 and 54 after it appears aren't understood; retail's spawn display flag 0x20 is left out.
* The premium saddlebag isn't offered; the saddlebag can be opened anywhere.

## Auracite backups

* Gear sets, hotbars and macros come back only through the plugin's settings-file restore (they live on the PC);
  backups made before kawari-diagnostics `babddba` don't have the files, nor the adventurer plate.
* Only the members in [Auracite Backups](auracite.md#what-each-importer-takes) are imported. Gear sets, retainers
  (no appearance in the backup to hire them with), achievements, hotbars, macros, tribal reputation, logs and records
  are only kept (`auracite_extended`) until Kawari has somewhere for them.
* A quest's flags and accepting class aren't kept (Kawari's `PersistentQuest` has neither); the export writes them as 0.
* ArchonEngine ControlPanel's character snapshots and restores don't cover the `auracite_extended` row.

## Settings backup

* The game's Server Backup (client settings and character configuration) isn't served: it uses its own TCP
  connection (`SaveDataBankHost`/`SaveDataBankPort`, default config-dl.ffxiv.com:55000), which Bob's Chronofoil
  captures of 2026-10-24/25 don't include. Its protocol is unknown until it's recorded with `tools/sdbrelay`
  ([Settings Backup](settings-backup.md)); until then gear sets, hotbars and macros come back through Auracite
  (Kawari)'s settings-file restore.

## Systems (upstream gaps the fork hasn't touched)

Duty completion isn't saved per player; housing doesn't persist
furniture; market board, trade, retainers, blacklist and fellowships are placeholders. See `FORK_ROADMAP.md`.

## The world server's two tasks (2026-11-05)

* The message loop and the tick no longer deadlock ([Timing](timing.md#the-worlds-turn-2026-11-05)), but the locks
  inside a step are still taken in either order (`data` then `network` in about 75 places, the other way in about 45);
  that's only safe while those two tasks are the only ones that take them, each holding the turn. A new task that
  touches the world must take the turn too. A step that never ends (a loop that doesn't finish) still stops the world:
  the watchdog only logs it.

## ArchonEngine ControlPanel

* Gil purchases only work while the character is logged out (by design, see [Saving](saving.md)).
* 162 costume sets in the Shop have no contents yet and can't be bought until an admin adds them.
* Store items arrive as items; what they do when used depends on server support.
* Packet captures (`data/captures`, one file per connection while capturing is on) have no limit; only Diagnostics'
  clear button removes them.
* The battle replay (2.25.0) was checked in a test ArchonEngine ControlPanel with an action index made from exported
  sheets (no Omen sheet there, so every cone came out 90°), not yet on Bob's server with `kawari-questdump actions` from
  his own install. Actors show by BNpcBase and level, not their names (no BNpcName list read from the game yet). A
  capture is read from Kawari's decoded text, so a packet whose Debug text changes shape (a renamed field) silently
  drops out of it.
* `!mark` and Kawari Probe 0.7.0's *why am I stuck* window are built and tested outside the game only (the Probe in CI,
  `!mark`'s parsing in a test); neither has been used in game yet. The Probe's three hooks (UseAction,
  ShowLogMessage, ShowLogMessageUInt) come from ClientStructs; a wrong one shows as failed in the window, the rest
  still read.

## Achievements

* Achievement items (the Achievement sheet's Item) are given as an obtained item with its log line; no retail
  completion with one was recorded, so the packets, and what happens with a full inventory (retail may keep it for
  Jonathas), aren't checked. Titles from elsewhere (quests, Grand Company ranks, PvP) aren't given.

## FATEs

* Entering a zone, Kawari sends FateInit before CreateFateContext; retail creates the contexts first (during the
  loading screen) and sends FateInit after the player has loaded in. Not known to break anything.
* What 2352 (`UnkFate2`, sent with the FATE start NPC's actor ID on leaving) does isn't known.
* FATE rewards ([Level Sync](level-sync.md#fate-rewards)) are gold rank's EXP and gil only, for whoever helped defeat
  its enemies; no Grand Company seals, items, ranks or the reward window (opcode 291 on 2026.09.15, 72 bytes, not
  known for 2026.10.06).
* Duty rewards aren't given: retail gives each boss's InstanceContent.BossExp (FinalBossExp for the last) times the
  player's bonus, NewPlayerBonusExp/Gil on a first clear, InstanceClearGil (LootMessage kind 9), and 10–12 gil per
  level 28 dungeon enemy.

## Combat (found 2026-10-17)

* Enemy area attacks now count a player half a yalm past their edge as caught (was a whole yalm); retail's exact margin
  isn't measured (Bob's retail captures have no area attack on him that he dodged).
* The blanga's timeline casts Terror Eye 6 s in and every 20 s, at its target; retail's cast it once in its 28 s, at
  the ground among the allies. One run; needs more before changing it.

## Packet layouts (from retail captures)

* **EffectResult / EffectResultBasic**: the HP is the packet's own actor's (retail sends each actor's result from that
  actor); the ID at 8, which Kawari calls `target_id`, is sometimes someone else (20 of 96 EffectResults in To Guard a
  Guardian).
* **To Guard a Guardian's masked mage**: retail spawns it not hostile and, when it engages, sends ToggleWeapon,
  SetEventState 0, SetBattle 1 and SetTarget. Kawari now spawns it not hostile and lets the first hit start it, but
  sends none of those: check in game that it can be attacked and its nameplate turns hostile.
* **Display flags 0x008 and 0x200**: To Guard a Guardian's scenery NPCs have them (no health bar, not targetable on
  retail); which bit does which isn't known (Kawari calls 0x008 `UNK2`, 0x200 has no name). They're copied as a pair.
* **The gargoyle's wait** in To Guard a Guardian is 5 s after it appears, retail's time; Kawari has no "player has
  loaded in" trigger for scripts, so a slow load could let it set off early.

## Packet layouts (from the client)

Confident findings from `kawari-opcodescan layouts` on 2026.10.16 that haven't been acted on yet. The client reads
bytes Kawari pads or calls `unk` in AcceptQuest/FinishQuest, Countdown, FriendRemoved, MarketBoardPurchase,
ZoneInit, AoeEffect8, UnkClassRelated, ContentFinderCommencing and Mount. The table is in
[Packet Layouts from the Client](packet-layouts.md#leads-confident-findings-worth-checking-next). Each needs the
handler read or a recording before a struct changes.

* 34 server and 5 client packets aren't mapped at all, mostly handlers reached through virtual calls.

## Packet sizes (from retail captures)

Packets whose size on retail isn't what `opcodes.yml` says, from Bob's Chronofoil captures (every version since June
2026; checked 2026-10-17). Kawari serializes each packet from its struct, so the effect depends on the packet:

* **EventScene255**: 1048 bytes on retail (2026.06.18, 09.15, 10.06), 1052 in `opcodes.yml`. Kawari sends it for
  scenes with 129–255 arguments (`event_scene.rs`): check the struct.
* **TitleList**: 120 bytes on retail (06.18–09.15), 112 in `opcodes.yml`. Kawari sends a 112-byte unlock bitmask
  (`TITLE_UNLOCK_BITMASK_SIZE`), so titles in retail's last 8 bytes probably can't show as unlocked.
* **DeepDungeonParty**: 24 bytes on retail (10.06, the Palace of the Dead), 20 in `opcodes.yml`; for the
  `Deep-Dungeon` branch.
* **StatusEffectListDouble**: 744 bytes on retail (about 22,000 packets over three versions), 720 in `opcodes.yml`.
  Kawari doesn't send it. Retail sent it for players only, never NPCs (20,537 for others, 2,221 for oneself, against
  StatusEffectList's 44,688, 10,080 and 37,901 for NPCs), and its first byte isn't a ClassJob as StatusEffectList's is,
  so its header differs and isn't worked out; which players get it (54 of 1,306 players only ever had it, 7 both) isn't
  either.
* **CrossRealmListingsRequest2** (client): 64 bytes on retail, 40 in `opcodes.yml`.

## Duty scripts against duty-sim (2026-11-10)

`questkit duty-sim` passes clean since its second pass (21 of 21 runs, `duties/known.json` empty;
[Testing](testing.md#duty-sim)). What's left from it, none of it played:

* The seven new or changed `onPartyWipe`s ([Dungeons](dungeons.md#at-a-wipe-duty-by-duty-2026-11-10)) were never seen
  on retail and are only rehearsed in the simulator. Not handled: a boss that has fallen while its add or the dragon
  still stands (Brayflox: no music at the next pull); Toto-Rak's last boss's tail model after a wipe; whether
  Brayflox's ledge goblin calls again.
* **Sastasha's captains run off** instead of dying (retail's, eight runs), by the simulator only: whether
  `retreat_bnpc` finds the way down the corridor on Kawari's navmesh isn't known (he's taken away 22 or 24 s later
  wherever he is, and the way on opens as he turns either way). What retail sends as he turns and when his coffers
  come wasn't read. Denn now comes only as someone walks into the cove's range (`EVENT_RANGE_BOSS`, the one that has
  played his cutscene): if that range didn't fire, he wouldn't come.
* **The Tam-Tara rooms' boss** comes as the last of the pack falls; in three of fifteen retail rooms it came with one
  still standing at 1-5 %, by a rule not found.
* `duty-sim` doesn't fight: enmity, hits and the party's NPCs aren't in it, and a boss retail pulled more than once
  isn't rehearsed. Event ranges are written into a fixture by hand (Sastasha's cove, at retail's time: it shows that
  the range brings Denn, not that the server fires it). `cargo test` (with `bundle_tests.rs`) runs on PRs, pushes to
  master and by hand, not on every push to `test-pending`.

## Bundle 20261110-182408 (the Stone Vigil, 2026-11-10)

Fixed in code and by test or simulation; none played yet.

* The toss: the one hit lands where they stood (retail's ActorControl 220 said so too). Whether the client shows the throw from kind 62 and 220 alone isn't known until it's played.
* "Bots don't attack during dodging": a waiting bot is given its target's position; there's no test (it needs the
  game's data).
* "Party bots normally wouldn't dodge but player bots should" was read as: Duty Support members keep their set
  spots, Player NPCs dodge by the shapes they see. If he meant something else, this is the place to change
  (`party_bot`'s `authored`).

## The Howling Eye, fleshed out (2026-11-11)

* What Garuda does after 661 (Aerial Blast again, the satin plumes, 661) is a guess: retail's one run that got there
  ended at 3 %. Eye of the Storm's damage (1061) is a guess: nobody was ever in its ring. 659 after a return comes
  once a phase as recorded; when it comes again isn't known (the pull's 49.5 s).
* A cracked monolith's threshold (65 %) is somewhere in 60.6-69.5 %, from four sightings.

## A duty's beginning (2026-11-11)

* A duty waits for every player's opening before it begins, and holds its enemies and the party's bots meanwhile;
  a player who never finishes it (a crash in the opening) holds the duty until they leave and someone else is ready.
* The party's cutscene states retail shows (ActorControl 519, the status icon 504) aren't sent.
* A razor plume of the Howling Eye whose monolith is already broken goes for the player (`set_hate` on someone gone);
  retail's choice then isn't recorded.
* "Moving sometimes stutters" on the airship (bundle 20261111-112150): nothing on the server's side shows it (every
  PingSync answered within a millisecond, no position sent to the player there.)

## Airship landings behind the ticketers (2026-11-11)

* The ticketers take you up to a landing only through a quest's own script: Lady of the Vortex (519) and Reclamation
  (520) before they're accepted, and the quests that list a ticketer in a sequence. Their pre-acceptance questions are
  from the client's scripts (unseen on retail). A quest not yet scripted whose NPC stands on a landing can't be reached
  from below until it's scripted the same way.

## Bob's marks of 2026-11-11 (quest battle 86, cast aims)

None of it played yet.

* Quest battle 86's talker engages and leaves the fight as retail's did (`engage_bnpc`); whether the client shows its
  red nameplate and drawn weapon from those ActorControls alone is untested.
* Casts are aimed as retail's for the 706 actions seen on themselves or on no one; the three seen both ways (120, 129,
  135) and every action not seen still go by CastType.
* The win's exit (territory 138, retail's pop range 4294805) has no fixture: `battle-sim` doesn't follow the player
  out.

## World bots (2026-11-11)

Stage 1 of [World Bots](world-bots.md); nothing seen in game yet.

* The places are found from each zone's layout: whether the marketboards' CustomTalk names (`CmnDefMarketBoard…`)
  and the innkeepers' Warps find every board and inn, and nothing else, is only known when the server logs them
  (`!worldbots` in the zone.)
* Retail's standing spots are kept for the six city zones only; elsewhere bots stand in a ring round their place.
* The shares of their trips, an inn's stay and a loading screen's length are guesses.
* No weapons drawn, minions, mounts or crafting yet; they don't turn to face their place after a walk.
* Sitting (2026-11-11): how often a bot goes to a seat or sits on the ground is a guess; seats are retail's 47 in the
  six city zones only.
* A spot must be reachable on the navmesh from the zone's landing: a zone whose navmesh doesn't join up (a raised
  walkway the navmesh lacks) puts fewer spots there; recorded spots aren't checked (retail stood there.)
* Friends kept from a world bot before 2026-11-11's second pass have the content ID of the actor it was then, so
  they're called as a copy, never found in the crowd.
* Searching for one, or a tell to one, finds no character.
* How long world bots wait after a client comes in (3 s) and how many come a tick (4) are guesses: if the zone's NPCs
  still load late in a busy city, they want longer or fewer.

## Raises, boss-fight spots and drop rates (2026-11-10)

None of it played yet.

* **Transcendent** (418) stops hits and ends at the raised one's first action; damage over time still ticks through
  it, and how long retail's lasts is the wiki's 5 s (nobody was raised in Bob's captures.)
* **Boss-fight spots** are where retail's party stood on the whole (medians by role), not each boss's own: a boss with
  attacks on its sides would catch the healer or the ranged there until they dodge. Duty Support members' authored
  spots (`party_spots`) still come first where a duty has them.
* **Mounts and minions** drop at 90 % (or the duty's rate), but no scripted duty's coffer lists one yet.

## Public sources (2026-11-10)

* `questkit sources --duty`'s drafts have no timing for dungeons before cactbot has timelines (A Realm Reborn's have
  triggers only), no damage, HP or levels, and the helpers' actions (the invisible 9020 actors') aren't placed: the
  README lists them for the director. A draft isn't a duty until a capture or Bob's run checks it.
* The Boss Mod reader takes shapes written in a component's constructor line; shapes built in code (rotating cones,
  shapes chosen by a status) are missed, and 4 cones and some rings it read wrongly are filtered by range
  (15-340 degrees; a hole smaller than its ring) rather than understood.
* `public_action_areas.json` is checked against the captures only where an enemy used the cone there: 29 of 838
  (`questkit chronofoil --areas`). Ring holes aren't checked.
* A draft made with `--captures` takes `every` as the median time between an enemy's uses of an action and the damage
  as a median over every run: phases (what a boss does below some HP), the order within a rotation, who it aims at and
  the adds' spawn times aren't worked out (`questkit battle` on the capture's `--battle` shows them). The helpers'
  spots are listed, not turned into director calls.

## The capture pipeline (2026-11-10)

* `--battle`, `--hits`, `--quest` and `--dropin` still read captures themselves, not the ledger
  ([the plan](capture-pipeline.md), stage B's "done when").
* The ledger reads the plain StatusEffectList only (StatusEffectList3, -Double and -PlayerDouble are skipped, and
  the Double is 9,290 to 715 times a version not its table's size), and keeps no `move` events.
* `--actions` has no MP cost (retail doesn't send one per use) and its shortest gap is a recast only for actions used
  back to back.
* `--controls`: 155 ActorControl categories and 31 ClientTrigger categories in the captures have no name in Kawari's
  enums (the tables are in the diagnostics repository, `facts/controls.json`); most of the plentiful ones are sent
  once a zone change (57, 361, 1207, 1208, 143, 1530, 943, 383, 573 to oneself, about 1,250 each in 123 captures).
* `questkit rotation` has no damage, so it can't say a rotation is good, only that it runs, is never idle and uses
  what retail's players use (`--against`); `--against` fails nothing.

## Build and tests

* `inventory::equipped::tests::test_item_level` needs a real game install, so `cargo test --workspace` fails without
  one.
* CI's "Quest replays and scripts" job runs questkit's own tests (`python -m unittest discover tools/questkit/tests`);
  run them before pushing. They failed from 2026-10-20 (`85124c4`) to 2026-10-21 (see the changelog) without anyone
  noticing. "Build and test" (the full Rust build) only runs on pushes to `master`, pull requests and by hand, so a
  branch's run being green doesn't mean the Rust side was built there.
* Running `cargo fmt` on the workspace reformats upstream files; only check the files you changed
  (`rustfmt --check --edition 2024 <files>`).
