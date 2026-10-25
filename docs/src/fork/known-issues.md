# Known issues and gaps

What is known to be missing, approximated or wrong in the fork. Remove an entry in the same commit that fixes it, and
add one when something is found.

## Upstream 7.57h1 (merged into `master` on 2026-10-21)

* Automatic opcodes after a game update ([Fork Opcodes](opcodes.md#every-opcode-after-a-game-update)) haven't run on a
  live update yet, only against Bob's 10.06 and 10.16 executables. About 16 server packets come from candidate lists
  (the notices, EventResume2/4/8, InitResponse and PingSyncReply, the inventory and market packets): a wrong pick shows
  as that packet doing nothing (10.16 → 10.06 also leaves three Crossworld linkshell packets among 12 candidates). 14
  client packets aren't found and wait to be learnt by size. `BASE_INVENTORY_ACTION`, the mail result codes and the
  scrambler tables aren't found automatically (no pattern found for them).
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

## Party NPCs (Duty Support)

* Untested in game (2026-10-23). Whether the client builds its party list from ActorControlSelf 257, 276 and 1525
  alone, as retail's capture suggests, is the first thing to check.
* Only Sastasha's, the Tam-Tara Deepcroft's and the Copperbell Mines' three members are recorded (Marauder, Lancer, Thaumaturge, for a
  healer player; the same three in both, at each duty's level, `party_npc::DUTY_LEVELS`); any other member chosen
  (their healer, other duties' members) is a Player NPC of that role, listed in a Duty Finder party list beside the
  party NPCs' slots. A duty not in `DUTY_LEVELS` gets them at Sastasha's level 15.
* The members' abilities that did no damage (26230, 29303, 29766, 29311, 29767, 25082, 25083) are only animations,
  and the Lancer's 29772 doesn't heal it. Their chatter in fights is at random every 150–400 s of fighting, not
  whatever made retail's members speak.
* The bots' diamond formation (also the Player NPCs') is measured with a healer player only.
* Getting out of area attacks (2026-10-24, `server/dodge.rs`) is untested in game, and so is bracing when there's nowhere to go (which actions retail's members used then isn't recorded; party NPCs brace with the player classes' actions by their class and level, not their kits). Only shapes `in_area` knows are
  dodged (circles, cones, lines and rectangles): donuts and crosses aren't read from the Action sheet yet, so nobody
  steps into a donut's middle (the third Tam-Tara room boss's 912 may be one). Every area attack a hostile NPC casts
  counts, even one that would do nothing to them; the margin past the hit edge (half a yalm) and the 0.8–1.3 s
  reaction come from one fight (Tam-Tara's last boss, two casts). A bot's own cast isn't interrupted: it slides it.
  The safe spot is checked only for standing on the navmesh, not for a path there that stays out of the area.
* How hard enemies hit them (2026-10-23) is the players' scale times retail's ratio for each member (a third for the
  Marauder), untested in game. On retail the enemies stayed on the tank (the Lancer and Thaumaturge took 330 damage
  each in 30 minutes); whether Kawari's enemies stray onto them more isn't checked.

## Party bots

* Stage 1 only (see [Party Bots](party-bots.md)): `!spawn playernpc` bots are in the party list only once invited,
  aren't level synced, don't follow you into another
  zone, use single-target attacks and one heal only. The Duty
  Finder doesn't fill roles with them yet. In combat melee bots still stand on the same spot by their target. Bots are
  spawned as retail's chocobo companions are (for their nameplates' names): whether the client treats them as your
  chocobo anywhere else (the companion's bar, its commands) hasn't been checked. They run at NPCs' speed (6 yalms a
  second), so they fall behind when you sprint or ride.

## Quests

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
  Limsa Lominsa). Kawari doesn't send it.
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
* Quest-spawned enemies (307's coblyns, 503's marshlights, 3853's goobbue) lack the quest fields retail's SpawnNpc gives them (Bob's
  capture of 2026-10-18): `event_npc_instance_id` = the triggering object's layout ID, `handler_id` = the quest,
  `owner_id` = the player (for 3853's goobbue, `event_npc_instance_id` is the EVENTRANGE that brought it out; and display flags 0x40003, which 307's layer doesn't set). The client script's
  GetGimmickState checks IsBattleNpcTriggerOwner, so the enemy's quest marker or claim may differ; the kills still
  count. Needs `spawn_quest_bnpc` to set them (Rust) and a trigger field in drop-ins.
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
* Event items: only the cast, the effect and the quest's event are done; the Casting condition retail sets during the
  cast (and clears after) isn't sent.
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

## Central Thanalan

* Some of its monsters still weren't fought on retail: the Sun Bat, Spriggans, Cochineal Cactuar, Antling Worker,
  Amalj'aa, Qiqirn Scrambler and the guards deal the generic 12 %, and the numbers that are there are from a
  gladiator (a tank). Skills seen but not scripted: the Huge Hornet's Sharp Sting, and those listed in combat.md
  ("Open-world spawns"). Only the Star Marmot's and Huge Hornet's drops are in `Drops.lua`.
* Western Thanalan (140) has retail's spawn points from one walk around it (2026-10-18), so spots Bob didn't pass
  near are missing; most of its kinds deal the generic 12 % (damage was recorded for six), none of their skills are
  scripted, and only the Sun Midge Swarm's drop is in `Drops.lua`. Not tested in game yet.
* Ovjang (the hunt mark) stands at one of its spawn points; retail moves it between them.

## Marketboard

* The Weapons and Armor tabs' sort (2026-10-23): which of the client's sort bytes (0x13, 0x12) is lowest level first
  is a guess, and the byte after it (6 on an armor category) isn't known; no retail capture has browsed those tabs.
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
  for 2026.10.06 isn't settled.) No approval quest check (retail's scene 1000).
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

## Coffers and loot

* Untested in game (2026-10-22): opening, the rolls, the winner's item, the coffer fading out. LootMessages name bots
  by their actor ID; whether the client finds their names (they're in the party list) isn't known.
* A Greed or Need on a unique item the player has already isn't refused (retail answers 8). LootItem's `bonus` (set
  on retail's Aetherial Brass Wristlets, perhaps their random stats) is left at zero.
* Only Sastasha's coffers have recorded contents; coffers 98 and 100 were never opened in the capture and hold one of
  the others' items. A reopened coffer's SpawnTreasure (retail sends it again with its items when you come back in
  range) isn't sent.

## Party bots

* The glamour list (`glamours.txt`) is untested in game (2026-10-23): whether the client shows an NPC's glamoured
  models and both dyes as a player's. Items limited to a race or a sex aren't checked, nor whether a glamour's items
  are ones its classes could wear.
* The Glamour Collector's page reader follows Eorzea Collection's markup as of 2026-10-23 (checked on 21 live pages,
  not yet through the installed extension); a site change will need it matched again. A glamour listing more than
  one job in "Intended for" wasn't seen, so whether those come as several icons is unknown.
* The glamour API (Kawari Web 2.21.0, `UI-Redesign` only) and the extension's Kawari Web connection were checked
  against a test Kawari Web in the workspace, not yet on Bob's server over the LAN or Tailscale. The server needs a
  build with this commit's `kawari-questdump classes` for the class list, and this commit's `KAWARI_DATA` path for
  the bots to see the list Kawari Web writes.
* The tank's spot across from the party and its keeping up with the player are untested in game.
* Leaving out gear and glamour pieces a bot's race or sex may not wear (EquipRaceCategory, 2026-10-23) is untested in
  game.
* Hiding a Viera's or Hrothgar's hat when its model has no file for their race (2026-10-23) is untested in game, as is
  whether a hat they can show always has its own model file (a shared one would be hidden wrongly). Bots keeping to
  the navmesh's floor where they walk straight is untested in game too; off the navmesh they still walk a straight
  line.
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

* Bosses' timelines (Chopper, the quarters' reaver, both captains, Denn, Denn's adds) are from six runs' medians
  (2026-10-22/23); their actions that did no damage (351, 1000) are only animations, and when the bosses first use
  each action isn't recorded (Denn's 556 comes about 14 s in, retail's 18). Denn's adds come by time from his first
  target (47 and 63 s), which fits the Duty Support run; retail's trigger isn't known (one Duty Finder run had its
  wave 71 s after he appeared).
* The second captain's gate fight and the clams' loop are untested in game (2026-10-23). Whether the four corsairs are
  in Sastasha's layout as Kawari reads it isn't known (Bob's earlier Kawari run never spawned them); if not, the
  server logs that it couldn't find them. ActorControl 503 and 11 (retail's, putting them on the enemy list) aren't
  sent.

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
* 28462's and 28760's effect (`kk 00 1b 00 00 00 00 80` then the action ID, `kk` per target: probably a knockback or
  draw-in) isn't decoded or sent. Retail's helpers' SetTarget on the player isn't sent. 28544's and 28549's damage
  are stand-ins (28546's): nobody was caught by them in Bob's three runs.
* The second boss's 14449 helpers are hostile (display flags 0x60008): held, they don't move or fight back, but
  players and bots may still be able to target and hit them.
* The last hall's trigger is a box around where retail's started (z below 55, y below -50) checked against the first
  player only (`info.player`).
* Statuses 8, 18, 53, 203 are only applied or nothing; the shortcut and the end's log 9393 aren't sent.
* Not done from the audit of 2026-10-25: the Limit Break gauge (retail's ActorControlSelf 505, 116 in the first
  stretch; Kawari never fills it); ActorControl 13 (3, value), sent from the player every ~6 s (its handler isn't
  identified); the second gigas pair (2790237, 4023509) pulled 5 s apart where retail linked them.
* Duty clear doesn't send retail's completion EXP and gil logs (9393 with 24200, 2163 with 2700), the director's
  Unknown7 (0x40000007: (0, 1) and (1, 1) during the run, (0, 0) and (1, 0) at the clear), ActorControlSelf 130 (3, 1)
  and 257 ×3, or UnkContentFinder2 (3, 4); the clear rewards need checking against the InstanceContent sheet.
* The timelines are by BNpcBase (1395-1403, 1421, 14448, 14449, 14460, 14537, 9020).

## Quest battles and directors

* **Into a Copper Hell (44)**: untested in game (2026-10-25). Territory 274's drop-in (`wil_w1t1`) appends a guessed LGB
  for Ul'dah - Steps of Nald; if 274 loads other LGBs, the three adventurers won't spawn. Retail's last DirectorVars
  (sequence 0xFE with its last data byte (9) at 0x80 as the closing cutscene ended) isn't sent that way; ActorControl 73 (1351,
  1352) on the pugilist now and then isn't sent; statuses 71, 99 are only applied.

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
* Underneath the Sultantree: the stranger's Flash, Sentinel and Sword Oath (seen on retail, none of them damage) have
  no action scripts and aren't in his rotation. The third pair of lesser blangas comes at 23 % of the blanga's health;
  in the one run it came between 24.2 % and 22.1 %, also 15 s after the second pair, so it might be timed. The blanga
  never went after the player on retail, so its hits on a player use the generic damage. Kawari ends the battle once
  everything is down (Sapphire's rule); on retail the blanga happened to fall last.

## Combat

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
  values, not checked at low level (only Conjurer/White Mage are in Bob's captures). No job gauges (BLM Astral Fire and
  Umbral Ice, MNK chakra/furies, WAR Beast, PLD Oath, SCH Aetherflow, NIN mudras; SMN's has Aetherflow only), no egis
  or fairy, no
  positionals, movement actions do nothing on the server, barriers don't absorb damage, Bloodbath doesn't heal, Cover
  doesn't redirect, Thrill of Battle doesn't raise max HP, Peloton is the caster's only. Berserk's and Perfect
  Balance's stacks aren't used up.
* **Combat items:** HQ values aren't used; food gives Well Fed without its stat bonuses; elixirs give their caps.
* **Cooldowns** (ActorControlSelf 17, retail sends one before every action) aren't sent; the client works recasts out
  itself. Item recasts aren't tracked.
* Retail adds up all of a target's HoTs into one tick (status 0) and shows MP ticks (Lucid Dreaming) as ActorControl 23;
  Kawari sends one tick per effect and no MP tick. Barriers' share isn't sent in EffectResult.
* Sprint: retail gives Jog (4209, +20 % speed until combat) when Sprint runs out, and keeps Sprint over a zone change;
  Kawari does neither (statuses are dropped on zone change). 8 of 37 retail Sprints lasted 10 s instead of 20 (all on
  the new 2026.10.06 characters), unexplained.
* Materia isn't counted in stats.
* No Limit Break gauge or Limit Break actions.

* **Teleport animation stayed on** after a teleport within the same zone (Bentbranch, 2026-10-16; Moraby Drydocks,
  2026-10-22): Kawari sent warp type 1 where retail sends 4 (see [Travel](travel.md#teleporting)); fixed 2026-10-22,
  untested in game. Retail's ActorControlSelf 263/236/253 around the warp still aren't sent.
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
* Not done: trial synthesis, collectables (collectability, CollectablesShop), food and potions, the completed-recipe
  log (CraftingLog is still sent empty: which bit a recipe is isn't known), specialist and Manipulation's quest unlock
  (the client greys those actions; the server doesn't check), crafting leves.

## Gathering

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

## Chocobo companion

* Untested in game (2026-10-24). Its fighting is from one capture at level 88 with Attacker at 6: how its HP and
  damage grow with level (ParamGrow's HpModifier is the stand-in), its skills' timing in Defender Stance, and the
  whole of Healer Stance (the Wiki's account) aren't recorded. Choco Guard does nothing (its status isn't known),
  Choco Surge doesn't raise healing, Choco Regen's ticks aren't scaled, Choco Medica heals one target, the
  Accuracy, Critical Hit and Attack Speed traits do nothing, and its attack range (3 yalms) is a guess.
* Companion EXP from a kill is a stand-in: as much as its owner's base EXP for it (retail's monsters well below Bob
  gave neither of them any). Retail's rank-up and onion packets aren't recorded (only CompanionUpdate is sent); the
  Challenge Log's Bosom Buddies, chocobo raising at a stable, the favourite feed, colouring, naming, its party-list
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
* Kawari Web's character snapshots and restores don't cover the `auracite_extended` row.

## Settings backup

* The game's Server Backup (client settings and character configuration) isn't served: it uses its own TCP
  connection (`SaveDataBankHost`/`SaveDataBankPort`, default config-dl.ffxiv.com:55000), which Bob's Chronofoil
  captures of 2026-10-24/25 don't include. Its protocol is unknown until it's recorded with `tools/sdbrelay`
  ([Settings Backup](settings-backup.md)); until then gear sets, hotbars and macros come back through Auracite
  (Kawari)'s settings-file restore.

## Systems (upstream gaps the fork hasn't touched)

Duty completion isn't saved per player; housing doesn't persist
furniture; market board, trade, retainers, blacklist and fellowships are placeholders. See `FORK_ROADMAP.md`.

## Kawari Web

* Gil purchases only work while the character is logged out (by design, see [Saving](saving.md)).
* 162 costume sets in the Shop have no contents yet and can't be bought until an admin adds them.
* Store items arrive as items; what they do when used depends on server support.
* Packet captures (`data/captures`, one file per connection while capturing is on) have no limit; only Diagnostics'
  clear button removes them.

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
  Kawari doesn't send it.
* **CrossRealmListingsRequest2** (client): 64 bytes on retail, 40 in `opcodes.yml`.

## Build and tests

* `inventory::equipped::tests::test_item_level` needs a real game install, so `cargo test --workspace` fails without
  one.
* CI's "Quest replays and scripts" job runs questkit's own tests (`python -m unittest discover tools/questkit/tests`);
  run them before pushing. They failed from 2026-10-20 (`85124c4`) to 2026-10-21 (see the changelog) without anyone
  noticing. "Build and test" (the full Rust build) only runs on pushes to `master`, pull requests and by hand, so a
  branch's run being green doesn't mean the Rust side was built there.
* Running `cargo fmt` on the workspace reformats upstream files; only check the files you changed
  (`rustfmt --check --edition 2024 <files>`).
