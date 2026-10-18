# Known issues and gaps

What is known to be missing, approximated or wrong in the fork. Remove an entry in the same commit that fixes it, and
add one when something is found.

## Quests

* **Close to Home** is only scripted for Conjurers (`ManFst004_00124`); the Lancer (`ManFst002_00085`) and Archer
  (`ManFst003_00123`) versions still stop after accepting.
* **The Gridanian Envoy** (507) and **Call of the Sea** (674) were written from Sapphire and the recording without the
  client's script: dump them in the Quest Lab to check the scene numbers. Envoy's arrival positions are Sapphire's.
* Story quests stop after **Call of the Sea** (674), which hands over to Limsa Lominsa.
* Limsa Lominsa has only upstream's opening. Ul'dah's story stops after **Underneath the Sultantree** (550), and its
  **Close to Home** is only scripted for the class Bob played (`ManWil002_00568`); the other two versions
  (`ManWil003_00569`, `ManWil004_00570`) are still upstream's stubs.
* Ul'dah's Close to Home: what Momodi's scenes 6 and 7 are for (talking to her during sequence 1?) isn't known, so
  she just ends the event then. Nothing to See Here's scenes 11–13 (the three NPCs' lines after the quest moves to
  255) are a guess from the listing.
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

* Most of its monsters weren't fought on retail yet: their damage to players is the generic 12 % and, where the
  captures didn't show it, their auto-attack is the default 870. The Huge Hornet's Sharp Sting (seen, no damage
  recorded) has no skill script. Their drops aren't in `Drops.lua`.
* Ovjang (the hunt mark) stands at one of its spawn points; retail moves it between them.

## Marketboard

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

* Not tried on Kawari yet; every opcode is confirmed from Bob's 2026.10.06 capture.
* The vocate's menu choices other than hiring and letting go (5 and 8 on retail) end the visit; hiring's yield 3 is
  answered with one 0, not retail's eight zeros on resume opcode 329 (upstream's EventResume8 is 513: which is right
  for 2026.10.06 isn't settled.) No approval quest check (retail's scene 1000).
* Partial gil withdrawals weren't captured: Kawari treats a 0 amount as "everything" and any other amount as an
  entrust.
* Retail sends a retainer's containers two or three times with an Equip in between, and ActorControlSelf 570/574 before
  their menu; Kawari sends the containers once. Scene 2's third number is guessed (5 − retainers hired).
* Retail placed the retainer at a spot set for each bell; Kawari puts them in front of the player. A new retainer is
  level 1 (retail 0) so the NPC has stats.
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
  animation locks are 0.6 s.
* **Raise** isn't implemented. Retail (117 revivals): the offer is status 148 (param 8, 60 s); accepting brings the
  player back where they are at 20 % HP and MP with Weakness (43, 100 s), or Brink of Death (44) if already weakened.
  Kawari's revive always warps to the entrance at full HP.

## Quest battles and directors

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
  Umbral Ice, MNK chakra/furies, WAR Beast, PLD Oath, SMN/SCH Aetherflow, NIN mudras), no egis or fairy, no
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

* **Teleport animation stayed on** after a same-zone teleport (Bentbranch, 2026-10-16 bundle): the warp itself was
  on time (PrepareZoning straight after the cast, ActorSetPos 2 s later, ZoneIn with 110 after FinishZoning), but two
  enemies that couldn't path to the player still had them on their enmity list throughout. With InCombat now set,
  the client won't start a teleport in combat, so this shouldn't come up again; if it does without enemies, it's a
  different cause.

## Connections

* Anything that keeps a player's connection busy for a second or so (a slow sheet read in a loop, say) desyncs the
  client's Oodle stream: every later packet from it fails to parse and the player is stuck (frozen, endless loading)
  until they reconnect. Why a delay breaks the stream, rather than only slowing it, isn't known.

## Items

* Items that teach **folklore** (4107) or **secret recipe books** (2136) aren't handled (the server doesn't keep those
  unlocks yet); they say "Unhandled item type".
* Learning something from an item has no log message of its own on Kawari; whether retail sends one (or the client
  prints it from the actor control) isn't known.

## Character windows

* **Achievements**: retail follows every `AchievementCompleted` (515) with ActorControl 518 and the same ID; Kawari
  doesn't send 518 (it isn't in `ActorControlCategory`), so what others nearby see when you earn one is probably missing.

* **Adventurer Plate**: a character who has never saved theirs gets upstream's starting design (expression, lighting,
  pose, borders, a made-up profile: active hours, play styles, preferred job 38) around their own look and gear;
  retail's starting plate hasn't been recorded. Once saved, it's the client's own. What the first number of
  UpdateAdventurerPlate (1) and the server's AdventurerPlate unk fields mean isn't known.
* **Achievements**: only Types 2, 3, 6, 8, 9 and 20 and counter 11 are completed automatically (of 33 Types and 1128
  counters), and what those Types mean is the community's reading of the sheet, not checked on retail. Mapping the
  Realm needs every part of the map discovered, and Kawari reveals map parts one at a time as the player walks into
  them (retail sometimes reveals a whole map at once, see `server/zone.rs`), so it depends on that working for every
  part of a zone. Nearly completed lists are always empty. Whether retail also sends
  LogMessage 952 ("… earned the achievement …") on completion, or the client prints it, isn't known.

* `!` debug commands are only recognised in zone chat (Say/Shout/Yell); in Party, Tell, Linkshell, Free Company or
  Novice Network chat they're posted as ordinary messages (upstream's behaviour).

## Systems (upstream gaps the fork hasn't touched)

Crafting is a stub; gathering uses placeholder EXP; duty completion isn't saved per player; housing doesn't persist
furniture; market board, trade, retainers, blacklist and fellowships are placeholders. See `FORK_ROADMAP.md`.

## Kawari Web

* Gil purchases only work while the character is logged out (by design, see [Saving](saving.md)).
* 162 costume sets in the Shop have no contents yet and can't be bought until an admin adds them.
* Store items arrive as items; what they do when used depends on server support.

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

* **HaterList and EnmityList**: on retail the entries start straight after the 4-byte count (HaterList: 264 bytes, 32
  entries of actor and enmity, then 4 bytes; EnmityList 72, 8 entries). Kawari's structs (`hater_list.rs`,
  `enmity_list.rs`) put 4 bytes of padding there instead, and Kawari sends both: the client probably reads Kawari's
  enemy list and enmity bars one field off.
* **EffectResult / EffectResultBasic**: the HP is the packet's own actor's (retail sends each actor's result from that
  actor); the ID at 8, which Kawari calls `target_id`, is sometimes someone else (20 of 96 EffectResults in To Guard a
  Guardian).
* **To Guard a Guardian's masked mage**: retail spawns it not hostile and, when it engages, sends ToggleWeapon,
  SetEventState 0, SetBattle 1 and SetTarget. Kawari now spawns it not hostile and lets the first hit start it, but
  sends none of those: check in game that it can be attacked and its nameplate turns hostile.
* **Void Call** (Quzal): scripted at 27 %; one run can't tell whether retail goes by health or by time after sequence 4.
* **Display flags 0x008 and 0x200**: To Guard a Guardian's scenery NPCs have them (no health bar, not targetable on
  retail); which bit does which isn't known (Kawari calls 0x008 `UNK2`, 0x200 has no name). They're copied as a pair.
* **The gargoyle's wait** in To Guard a Guardian is 5 s after it appears, retail's time; Kawari has no "player has
  loaded in" trigger for scripts, so a slow load could let it set off early.

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
* Running `cargo fmt` on the workspace reformats upstream files; only check the files you changed
  (`rustfmt --check --edition 2024 <files>`).
