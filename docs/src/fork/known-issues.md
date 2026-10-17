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

## Player regeneration

* Retail gives a point or two of HP a tick in combat (a level 15 conjurer: +1 or +2 every 3 s); Kawari gives none. Out
  of combat Kawari gives 10 % a tick, which hasn't been measured against retail.

## FATEs

* Entering a zone, Kawari sends FateInit before CreateFateContext; retail creates the contexts first (during the
  loading screen) and sends FateInit after the player has loaded in. Not known to break anything.
* What 2352 (`UnkFate2`, sent with the FATE start NPC's actor ID on leaving) does isn't known.

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
