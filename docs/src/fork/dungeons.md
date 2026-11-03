# Dungeons

The duties the fork has scripted from retail captures: their director script (`resources/scripts/content/<short
name>.lua`, the ContentFinderCondition's ShortCode), their enemies (a drop-in, `resources/dropins/<short
name>_planmap.json`: the client's layouts have no battle NPCs), the enemies' timelines (`resources/timelines/`) and
action scripts. The director API they use is described in [Quest Battles and Directors](quest-battles.md); the Duty
Support members in [Party NPCs](party-npcs.md); coffers in [Coffers and Loot](loot.md).

| Duty | ContentFinderCondition | Territory | Script | Recorded |
|---|---|---|---|---|
| Sastasha | 4 | 1036 | `content/s1d1_re.lua` (upstream's, with the fork's changes) | five Duty Finder runs (2026-10-22), one with Duty Support (2026-10-23); an arcanist's Duty Support run (cc62ad94, 2026-10-28: the Conjurer's lines, [Party NPCs](party-npcs.md#talking)) |
| The Tam-Tara Deepcroft | 2 | 1037 | `content/f1d1_re.lua` | one run with Duty Support and two Duty Finder runs (2026-10-24) |
| The Copperbell Mines | 3 | 1038 | `content/w1d1_re.lua` | one run with Duty Support and two Duty Finder runs (2026-10-25) |
| The Bowl of Embers (Ifrit) | 56 | 1045 | `content/w1fa_re.lua` | one run with Duty Support, two attempts (2026-10-27) |
| The Thousand Maws of Toto-Rak | 1 | 1039 | `content/f1r1_re.lua` | one run with Duty Support (caa5e908) and two Duty Finder runs (1fac2416, 2528a32d), 2026-10-30 |
| Haukke Manor | 6 | 1040 | `content/f1d2_re.lua` | one run with Duty Support (5d9c7d7f) and three Duty Finder runs (e7580e0d: twice as the tank, once as the healer), 2026-11-02 |

## Facing at the entrance

Retail turns players the same way every time they come into a given duty. Kawari had them face the way the
InstanceContent's entrance rect faces (or, without one, the entrance circle), and that was wrong in most duties: the
Thousand Maws of Toto-Rak 0 where retail turned them to 1.5708 (Bob's bundle 20261030-165554), the Copperbell Mines
1.5708 for 1.7881, Sastasha -1.4752 for -1.6239; the Tam-Tara Deepcroft's 0 happened to be right. Why the rects
differ from retail isn't known (there's no game data here to look at them with).

So retail's facing is recorded, by territory, in `resources/data/entrance_facing.json` (`entrance_facing.rs`): 65
territories from Bob's Chronofoil captures (June to October 2026), each either the ActorSetPos that put the player at
the entrance or the rotation in the client's first position update, which came about a second after the zone loaded
and before the player could turn. Every duty entered more than once had the same facing each time (Sastasha 8 times,
the Copperbell Mines 6, Toto-Rak 3). Six whose first update came more than 3 s in were left out (378, 522, 523, 580,
581, 838); Haukke Manor (1040) is -1.5705, the ActorSetPos of one of its four runs (Bob's capture e7580e0d of
2026-11-02; the other three's first updates came 8.7-22 s in). What decides the facing, in order (`change_zone_warp_to_entrance`, coming in and
after a wipe): a duty script's `ENTRANCE_ROTATION`, the recorded facing, the entrance rect's, the entrance object's.
The server log says which: "Players come in facing … (retail's, recorded for …)" or "The entrance rect … faces …".

## Clear rewards

What a cleared duty gives (`ZoneConnection::duty_clear_rewards`), as retail did in 14 clears of Sastasha, the Tam-Tara
Deepcroft, the Copperbell Mines and the Bowl of Embers (Bob's Chronofoil captures c99fe374, cc62ad94, 700cc294,
3ec702a8 and f5e78b52, 2026-10-20 to 10-28), all from the duty's director and in this order:

1. **The player's own first clear**: the clear EXP, the InstanceContent column after FinalBossExp (unnamed in the
   sheet: Sastasha 18,800, Tam-Tara 19,600, Copperbell 24,200, Haukke Manor 39,000; the Bowl of Embers 0), then
   EventLogMessage 9393 with it.
2. **The first-timer bonus**, on the first clear or whenever someone in the party hadn't cleared it (the arcanist's
   second Copperbell, a Duty Finder party, had the bonus without the clear EXP): NewPlayerBonusExp, then
   EventLogMessage 2163 with NewPlayerBonusExp and NewPlayerBonusGil, then the gil's slot (Sastasha 2,100 EXP and
   2,500 gil; Tam-Tara 2,200 and 2,600; Copperbell 2,700 and 2,700; Haukke Manor 4,300 and 3,800; the Bowl 90 and
   1,500).
3. A repeat clear with nobody new gave neither (ba1b86c5's two Copperbell runs, cc26e0ea's two Tam-Taras).

InstanceClearGil (400 for the three dungeons) and InstanceClearExp were never given. Each player's connection says
whether they're new as they enter (`ToServer::DutyNewcomer`, `Instance::newcomers`; party NPCs and bots never are), the
clear tells everyone (`FromServer::DutyCleared`'s second value), and the duty is then remembered as cleared: the
cleared bitmasks in the character's content data (`set_content_cleared`, indexed as upstream's unlocked ones: dungeons
from InstanceContent 1, guildhests from 10001, trials from 20001, raids from 30001), which the client is sent at login.

**Enemies' gil**: in a duty with another player there, each of the zone's own enemies (spawned as the instance was
made, `Instance::layout_npcs`; not the bosses, adds and waves the director spawns) drops gil for each player it
credits: its level × 0.400-0.532, rounded down (`actor::kill_gil`), after the kill's EXP, as the gil's slot,
LootMessage 9 with the amount and ActorControlSelf 42. Retail: 133 of 133 such kills in the Duty Finder runs dropped
(level 15: 6-7 gil, 16: 6-8, 17: 6-9), none of the 83 boss, add and wave kills, and none of the 208 kills in Duty
Support runs (no other player) ([Coffers and Loot](loot.md#enemies-gil)).

## The Tam-Tara Deepcroft

From Bob's Chronofoil captures of 2026-10-24: `3ec702a8` visit 0.3 (23 minutes with Duty Support, during Fire in the
Gloom, quest 677, which unlocks it) and `cc26e0ea` visits 0.2 and 0.4 (Duty Finder runs with three other players),
read with `questkit chronofoil --battle` and the visits' packets (numbers only).

**Enemies.** `questkit chronofoil --dropin 1037 --appends bg/ffxiv/fst_f1/dun/f1d1/level/planmap.lgb` gave the 74
battle NPCs seen (level 16; trash 980 HP). The rooms' bosses (BNpcBase 567), the last boss (76) and its adds (125,
126, 127) are `nonpop`: the director spawns them. Every kind has a timeline from the three runs (`TamTara*_<base>.json`:
retail's damage to players and to the Duty Support members, rotations); the casters (1345, 1347, the packs' leaders,
the last boss) cast their 0.7 s spell every 3.1 s instead of auto-attacking. New action scripts: 987, 988 (the last
boss), 911, 912, 430 (the rooms' bosses), 720 (trash 1388).

**The rooms.** Three rooms, each with a pack standing around a leader (82, 83, 137: 516 HP, casting 966 where they
stand) with two, three and four casters (1345). Retail, in all three runs:

1. A pack member going for someone: EventLogMessage 2012 (the room's PlaceName 535 / 539 / 541, 15), the battle
   music (director 0x80000001, 37) and the leader's NpcYell (563, 564, 565); 15 s later the wall shows (SetEventState
   0) with log 2013. The walls (2001699, 2001701, 2001703) spawn hidden (state 7); their lines (2001700, 2001702,
   2001704) visible. Kawari: `director:boss_room(boss, wall, line, place, pack)`, a boss arena sealed by any of the
   pack's pull (new; `spawn_boss` seals only on the boss's own).
2. The leader's death (HP 0) calls the room's boss (567, 4128 HP): log 2009 (its BNpcName 455, 1) and it goes for the
   party at once. In one run of nine it came 5 s after its leader died; the rest within a second.
3. The boss's death: both wall and line hidden, log 2014, the music off, log 2006, and its coffers: 118, 15 and 16
   after the first room, 17 after the second, 62 after the third (every run).

The rooms' bosses cast 430 on themselves as each fight starts and every 33 s (status 208, 100, 30 s), 911 at 8 and
18 s, and 912 (hitting nobody) at 22 s (`TamTaraMiniboss_567.json`'s timepoints).

**The gimmicks** (GimmickAccessor events, 0x000F00xx; each a 2 s interaction, then the event finishes; something to
pick up, a GimmickAccessor of the Interactable kind, has no prompt, and a prompt is scene 1 with flags 0x2001, see the
Copperbell Mines below):

| EObj | What | Retail |
|---|---|---|
| 2000061 | After the first room | opens once its boss is dead: hides the barrier 2001697 and itself, log 1537 |
| 2000062 | After the second room | the same, barrier 2001698 |
| 2000057 | The key (far side) | log 2031 (item 2000244), hidden |
| 2000060 | The third room's gate | a yes/no prompt; with the key: log 1536 (0), hidden, and the pack behind it comes at once (0.4 s), as a pull |
| 2000063 | After the third room | locked (log 2005, the event finishes refused) until its boss is dead; then log 1537 |
| 2000067 | The last room's seal | logs 1537 and 1539, hidden; the cutscene (scene 3 with 26, 1, 38, 1, 0) and the last boss appears 2 s later |

The director's variables (DirectorVars, sequence 1 throughout): 62, 0, 0, 0, 0 at the start; 60, 0, 0, 1, 0 after
door 1; 56, 0, 0, 2, 1 after door 2; 56, 1, 0, 2, 3 with the key; 56, 1, 1, 2, 3 after the gate; 48, 1, 1, 3, 3 after
door 3; 1, 1, 1, 4, 7 at the seal. The script sends the same.

**The last boss** (76, 6329 HP; arena wall 2001741, line 2001085, PlaceName 543, sealed 15 s after its pull as with
`spawn_boss`): it casts 971 every 3.1 s, 988 (a hit with a small drain) every 26 s, and 987 (a 4.7 s cast, status 17
for 20 s on its target) first as it falls to 57-60 %, then 63 s later. Adds: one shield (125) at 79-80 %, a shield and
three adds (126 ×2, 127) at 57-60 %, two shields and four adds at 28-29 % (all three runs). It took no damage while a
shield stood (its HP held for the 10-15 s each one lived): `director:set_invulnerable` (new) while any 125 is alive. Each shield is tethered to the boss: the line between them is no packet of its own but the shield's SpawnNpc, whose
tether target is the boss and tether ID (Channeling row) 1, in all three runs; it goes when the shield dies, with no
other packet (`director:spawn_tethered_bnpc`). Nothing else shows the shield on the boss: no status, VFX or director
variable while one stands. No other tether was sent in the Tam-Tara Deepcroft (nor any ActorControl tether, category
35).
The shields never act nor move. Its death: the arena opens, cutscene 27 (1, 18, 0, 0), coffer 11, the duty complete.

**Coffers** (`treasure_loot`): 16 held 3556 or 2679; 118 3330; 17 3555, or 3555 and 4310; 62 3033 or 3331; 11 31426,
3787 and 31425, or 31427. 15 and 119 (119 stands by the key from the start) were never opened.

**Duty Support.** The same three members as Sastasha's (DawnContent 201, members 36, 38, 37; ContentFinderCondition 2),
at level 16 with 443, 347 and 316 HP (`party_npc::DUTY_LEVELS`), with the same kits. Their story lines: the Lancer's
14307 34.5 s in and 14309 as the first room's leader falls, the Thaumaturge's 14311 as its boss falls, the tank's
14312 26.5 s after the last seal is broken. The Marauder's chatter included 13953, now among its lines. With a player
tank (Bob's capture c99fe374 of 2026-10-28, Dags, visit 0.9): the Conjurer (39) in the tank's place, the same lines
from the same members but the last, the Conjurer's 14313 26.8 s after the cutscene; the Conjurer fell once (514 s in).

**Not here:** the shortcut (2000700, shown on retail as the first room is pulled: its pop ranges need the game's
layout files to find), log 2008 (near the key, a rect's), what statuses 208 and 17 do, the BGM of the last fight.

## The Copperbell Mines

From Bob's Chronofoil captures of 2026-10-25: `bd298651` visit 0.5 (21 minutes with Duty Support, during Into a Copper
Hell, quest 660, which unlocks it) and `69bea0b9` visits 0.1 and 0.3 (Duty Finder runs with three other players, 10
minutes each), read with `questkit chronofoil --battle` and the visits' packets (numbers only). The script's name,
`w1d1_re`, is the ContentFinderCondition's ShortCode by analogy with `s1d1_re` and `f1d1_re` (not checked against the
sheet), and the drop-in's LGB `bg/ffxiv/wil_w1/dun/w1d1/level/planmap.lgb` likewise.

**Enemies.** `questkit chronofoil --dropin 1038` gave the 42 battle NPCs seen (level 17; trash 1051 HP). The bosses
(14460, 14448, 14537), the two ambushing gigas (1398, 1403, 2103 HP) and the second boss's adds and the first and
last bosses' helpers (14449, 9020) are `nonpop`. Every kind fought has a timeline from the three runs
(`Copperbell*_<base>.json`, retail's damage to players and to the Duty Support members); the casters (1397) cast 968
(0.7 s) every 3.1 s. New action scripts: 399-402, 421, 601, 677, 913, 914, 989, 6499 (the last boss's auto-attack),
28461-28465, 28543, 28545-28547, 28549, 28760, 28762-28764; the self-buffs (402: status 8 at 226; 421: 203 at 150; 400:
53 at 50) go on the caster (`gain_effect_self`).

**Coming in.** Retail turned players to 1.788 at the entrance (all three runs, about (-254.4, 23.8, -170.7));
Kawari turned them to 0, the way the entrance circle's own layout object (2000182, bound to 4085080) faces, which is
how every InstanceContent was entered (the Sastasha and Tam-Tara runs in Bob's bundles too; Tam-Tara's retail
facing happens to be 0). Players then faced the way the InstanceContent's entrance rect does (its LGBEventRange
column, an EventRange in the layout; Sapphire's `movePlayerToEntrance`), which gave 1.5708, still not retail's; since
2026-10-30 they face retail's recorded 1.7881 ([Facing at the entrance](#facing-at-the-entrance)).

**The way through.** Retail's DirectorVars (sequence 1 throughout): data 1 (the first byte after the flag and branch,
`set_data(0, …)`; the script wrote each one byte higher, so the Duty Information window counted nothing, Bob's bundle
of 2026-10-25) is the progress (1 past the first door, 3
the second, 7 the third), data 2 whether the party holds a charge (item 2000441), data 3 how many of the doors' items
(2000442) it holds. Every gimmick is a GimmickAccessor event (0x000F00xx) with a 2 s interaction (a yes/no prompt
first for the rocks, detonators, slots and doors), then the event finishes:

| EObj | What | Retail |
|---|---|---|
| 2000159 | The first charge | the last of the first pack (4021731-4021733, whichever falls last: a 1395 in two runs, a 1396 in one) drops it: log 2069 (its BNpcName, 2000441), the EObj spawned where it fell (layout 0); picked up: log 2031 (2000441), hidden, data 2 = 1 |
| 2000160 | The first rock | with the charge: log 2059 (758), hidden, data 2 = 0 |
| 2000163 | Its lever (gimmick 79, layout 4055428; a yes/no prompt, 0 for yes) | ActorControl 413 (8, 2) on the charge set by the rock (2000161, layout 2790162), 2001559 shown, the lever (2000163) and 2000165 hidden; 10 s later 2000162 and 2000166 shown, 2001560 hidden: the way is clear |
| 2000169, 2000172 | Items by the first door | log 2031 (2000442), hidden, data 3 + 1 |
| 2001536 | The first door's slot | refused (log 2030) until both are held; then log 2060, hidden, data 3 = 0 |
| 2000170 | The first door | refused (log 2029) until the slot is used; then it and 2000171 hidden, data 1 = 1, log 2061, and the first boss appears |
| 2000178 | The second charge | shown as the first boss falls; as the first |
| 2000173, 2000175 | The second rock and its lever (gimmick 107) | as the first: 413 (8, 2) on 2001529, 2001561 shown, 2000175 and 2000176 hidden; 10 s later 2000174 and 2000177 shown, 2001562 hidden |
| 2001531, 2000179 | Items dropped by the two gigas (1400: 2790234, 2790237) | log 2069 (1303, 2000442) as each falls, spawned where it fell; picked up as the others |
| 2001537, 2000180 | The second slot and door | as the first; the door: 2000181 hidden, data 1 = 3, log 2065, the second boss appears |
| 2001532, 2001533 | Items by the third door | the second shown as the second boss falls |
| 2001538, 2001534 | The third slot and door | as the first; the door: 2001535 hidden, data 1 = 7, log 2062 |

Retail asked nothing for the pickups (the charges and the doors' items: EventStart, then 1.9 s later the log and
EventFinish, no EventScene2) and showed scene 1 with flags 0x2001 (HIDE_HOTBAR and NO_DEFAULT_CAMERA) for the rocks,
levers, slots and doors. Kawari asked yes or no for every one (flags 0x2000). Now a GimmickAccessor of the Interactable
kind goes straight to the director as it's talked to (`StartTalkEvent` in `main.rs`), and the others show retail's
flags (`GimmickAccessorEventHandler`); Sastasha's and the Tam-Tara Deepcroft's keys are pickups too. The refusals' logs are the gimmicks' own (`isGimmickFunctional`). The first lever was scripted as 2000161, which nobody can use: saying yes to its prompt did nothing and the run was stuck there (Bob's bundle of 2026-10-25); the EObj each gimmick event came from in all three runs is the one in this table. The EObjs hidden at the start (state 7 in their
first spawn) are hidden in `onSetup`. **Ambushes:** the two bigger gigas (1398 at layout 4023421, 1403 at 4023560) wait
in a pile of rock (2000168, 2000185); someone coming within 19.4-22 yalms (all three runs) hides the pile and the gigas
comes at them at once (it spawned and targeted the player in the same packet burst, display flags 0x4002B with
INVISIBLE, 0x20). 0.58-0.94 s later (all six ambushes) retail brought it into view: ActorControl 36 (1, 158) then 54
(1). The client's ActorControl handler (2026.10.16) says what those do: 36 flags the object for an update, fades it in
over a second (its first parameter) and plays ActionTimeline 158 on it (its second); 54 makes it targetable. Kawari
does the same (`director:appear_bnpc(layout, 158)`, `ActorControlCategory::Appear` and `SetTargetable`); spawned with
INVISIBLE and nothing after it, the gigas had fought the party unseen (Bob's bundle of 2026-10-25). **The last hall:** the cutscene (scene 3 with 178, 1, 38, 1, 0) began as the first player
came down past z 55 (52-56 in two runs; players waiting at 58-63 didn't start it), the boss 2.4 s later, and
ActorControl 409 (64, the director) on EObj 4217260; there was no gimmick event.

**Kottos' leaps** (the first boss, 14460; all three runs alike): three invisible helpers (BNpcBase 9020, all layout
9054923, battalion 4) come with it and stand at the arena's middle (43, −9.27, −89.56). As Kottos begins 28543 (7.7 s,
on no one, 0xE0000000) they're put (ActorSetPos, all in one tick) on three of four spots 7 yalms north, east, south and
west of the middle: the first on north or south, the second on east or west, the third opposite the first (N-E-S,
S-W-N, N-W-S in the runs). Each casts 28544 on itself there (8.7 s), 2 s apart. As each one goes off (9.0, 11.0 and
13.0 s after 28543 began, hitting nobody in any run) Kottos lands on its spot: ActorControl 239 (`SnapNextMove`: the
client's handler sets a flag in its MoveController so the next move is a snap) and an ActorMove there (speed byte 90,
y −9.78), 9.1, 11.2 and 13.3 s in; 28549 (instant, hitting nobody) 1 s after the first two landings, and 28546 (3.7 s,
on itself) 1 s after the third. The helpers go when it dies. Kawari: `onActorCast` in `w1d1_re.lua` with the new
director methods (`spawn_bnpc`'s actor IDs, `warp_bnpc`, `actor_use_action`, `jump_bnpc`, `delay`); 28549's and
28544's damage is a stand-in (28546's, 44.5) since nobody was caught by either (`CopperbellHelper_9020.json`).

**Kottos' schedule** counts from its pull, the same to a tenth of a second in all three runs however long it had
stood before (5.2–10.0 s after it appeared): auto-attack 870 at 3.0 s and every 3.0 s when it isn't busy; 28545 (4.7 s,
the tank, lock-on 218) at 5.2; 28543 at 15.3 (its leaps above; no auto-attack or walking from then until 28546 lands);
28546 (3.7 s, round itself) at 29.6; 28547 (4.7 s, on someone other than its target at random: a lancer, a healer, a
ninja) at 39.1; 28543, 28546 and 28547 again 37 s later (52.3, 66.6, 76.0); then 28545 at 86.2 and 28543 at 99.4, so
from 86.2 the round repeats every 81 s with 13.2 s between its 28545 and 28543 (the first round's were 10.1 apart;
past 81 s only the Duty Support run, 101 s, shows it). Kawari: `CopperbellBoss1_14460.json` with `from_pull` (new:
the timeline counts from the pull to the half second, each point fires when due whether its target is in reach or
not, a point can repeat on its own `every`, and an action can go on its target, `other`, `itself` or `nobody`); 28549
is the director's, 1 s after each of the first two landings; `busy_bnpc` keeps it from acting on its own from 28543
to 28546. Before (Bob's bundle of 2026-10-26) its timeline counted from its spawn and only fired when it was in reach,
so it opened with 28543 and leapt nowhere (28549 came on its own); it swung its auto-attack as it began a cast, which
replaced the cast's record, so it walked and turned through its cast bars and its areas went off facing wherever it
had turned, not where they were drawn; its tank bot circled through it (sent round to a spot beyond its limit from the
player and back, every second), turning it; and the first marker's area panicked the server (an EObj asked for its
CommonSpawn).

**Which way Kottos' attacks go** (from the three runs, ActorCast rotations and who was hit; the names from the
[Console Games Wiki](https://ffxiv.consolegameswiki.com/wiki/Copperbell_Mines)): 28545 (Grand Slam) is the
tankbuster on its target. 28546 (Colossal Slam), cast on itself, is a large cone the way it faces as it begins, and it
faced its target, the tank: −1.94 with the tank at −1.94 and 1.04 with the tank at 0.97 (Duty Support run), hitting
the tank 8.7–10.5 yalms off and nobody beside or behind it; in the Duty Finder runs the players stood out of it.
28547 (Catapult) hits only the one it's thrown at, someone other than its target at random: the Marauder standing at
the same spot (5.0 yalms, 0.03 rad apart) wasn't hit, twice. Kawari's 28546 kept the facing its third leap had left
it (exactly −45° or 135°), so it went the same way every time and caught nobody, and the bots never had to get out
of it (Bob's bundle of 2026-10-26: "his cone AOE doesn't target anyone and only goes in one spot"). Now a timeline
action on `itself` faces its target as it begins (`npc_behavior`), and every NPC hit goes out facing the way its cast
was aimed (`action.rs`), not the way it turned in the tick its cast ended (its Catapult's hit swung back to the tank
there).

**The bosses** (`spawn_boss`: sealed 15 s after their pull, log 2012 (PlaceName, 15) then 2013; opened at their death,
log 2014; the battle music, 37, at the first two's pulls and off at their deaths):

| Boss | PlaceName | Wall / line (layout IDs) | Retail |
|---|---|---|---|
| 14460 (9053767, 9970 HP) | 679 | 9085524 / 9085538, and 9085522 hidden at its death | 28545 (4.7 s) at 2 s and every 81 s; 28543 (7.7 s, hitting nobody, with its three helpers' 28544) then 28549 twice, 28546 (3.7 s) and 28547 (4.7 s) every 37 s, the same to a tenth of a second in all three runs. Coffer 52 |
| 14448 (9040077, 11383 HP) | 680 | 9089361 / 9089385, and 9089366 | From its pull: 28462 (3.7 s) at 6.2 s, 28461 on the tank at 17.5 s (lock-on 218; only when no 28463 has come first, the Duty Support run); 28463 on itself (2.7 s) as its health falls to 80, 50 and 25 % (75.5-76.9, 42.6-49.8, 14.8-24.6 % as it began: 13.1-25.5 s in, its timing different in each run), each followed 8.6 s later by its six helpers' 28465 (5.7 s, 121-127; see below) and 22.3 s later by 28462 (if no 28463 has come by then). Kawari: `hp_below`, `restarts` and `after` points (new). Coffers 53 and 59 |
| 14537 (9060676, 10582 HP) | 681 | 9082207 / 9082214 | From its pull: 28762 (on itself) at 8.2 s; from 22.0 s every 47.7 s 28760 (on no one; the middle, its five helpers' 28761, 127, see below), 28762 14.2 s later, 28764 on the tank 12.1 s after that (lock-on 218), 28763 (on itself) 10.3 s after that, the same to a few tenths in all three runs. Cutscene 179 (1, 18, 0, 0), coffer 51, the duty complete |

The walls share their EObj ID with a line (2001526 for the first boss's, 2007457 the second's, and 2002735 is a line
of both the second's and the third's), so the script names them by layout ID: `hide_eobj`, `show_eobj`,
`delete_eobj` and the boss arenas now take either (`Instance::find_object_by_eobj_or_layout_id`: the EObj ID first).
`director:eobj_control(eobj, category, params)` (new) sends an EObj an ActorControl Kawari has no name for (413 for the
blasts, 409 for the last hall).

**Coffers** (`treasure_loot`): standing from the start 65 (never opened), 57 (3554) and 58 (3033 or 3035); 52 held 3554,
3554 and 4310, or 3555 and 4310; 53 3034, 3033 or 3035; 51 31423, 3786, 31428 and 33130, or 3781 and 31427; 59 was
never opened.

**Duty Support.** The same three members at level 17 with 476, 372 and 339 HP (`party_npc::DUTY_LEVELS`). Their story
lines: the Thaumaturge's 14315 40.3 s in, the tank's 14318 6.8 s after the first boss appears, the Lancer's 14320 4.8 s
after the second blast clears the way, the Thaumaturge's 14322 as the second boss is pulled, the Lancer's 14324 29.4 s
after the last boss's cutscene begins. Chatter: 13962, 13964, 13972, 13974, 13953, all already among their lines. With a
player tank (Bob's capture c99fe374, visit 0.14): the Conjurer in the tank's place, and the Thaumaturge's 14319 6.2 s
after the first boss appeared instead of the tank's 14318; the rest as before.

**The second boss's lines** (14448; all three runs alike): its six helpers (BNpcBase 14449, all layout 9040086, 369
HP, never fought nor killed) come with it. Each 28463 (2.7 s, on itself) lands 3.0 s after it begins; 1.0 s later the
boss grows (ActorControl 48, ModelScale 16, then 17, 18: the client's handler sets ModelContainer.ModelScaleId and
multiplies the base scale by the row's) and the six are put on its middle (ActorSetPos to (27, −37.96, 114)); they use
28464 (instant, an AoeEffect8 on no one) facing the way they'll cast, 0.6 s later leap to their spots (ActorControl
239 and an ActorMove, speed 0), and 4.0 s after that cast 28465 (5.7 s, a line, 121-127 to whom it caught) facing
outward. Two stand on a diagonal, (14.27, 126.73) facing −0.785 and (39.72, 101.24) facing 2.356, or (39.72, 126.73)
facing 0.785 and (14.27, 101.24) facing −2.356; four in a row at z 113.97 (x 45.0, 32.97 facing east, 20.98, 8.99
facing west) or a column at x 26.99 (z 131.98, 119.98 facing north, 107.99, 95.99 facing south). Each round picks the
diagonal and row-or-column at random (A+row, B+row, B+column in the Duty Support run, A+column, B+row in the Duty
Finder runs). They set their target on the player 1.3 s after the leap (not sent here). With the boss they go (no
ActorControl 417 for these). 28462 (3.7 s) is cast on no one at a point on the ground near a player (Kawari aims it
where its target stands).

**The last boss's lines** (14537; all three runs): its five helpers (9020, all layout 9060684) come with it. Before
each 28760 (5.7 s, on no one) it walks to the arena's middle (−100.41, −57.79, 6.67), and as it begins they're put
(ActorSetPos) facing the middle: four 14 yalms out, on an axis (north-south or east-west) and each end of it turned 60
degrees the same way, and one 7 yalms out across the axis on the side the turn leaves open (0, 60, 180, 240 with 270;
90, 150, 270, 330 with 0; 0, 300, 180, 120 with 90, in degrees from north: Duty Support's two rounds and both Duty
Finder runs). 3.0 s later they cast 28761 (5.7 s, a line, 127 to the one player caught); 1.1 s after 28760 lands the
boss gets ActorControl 239 and an ActorMove where it stands, and goes back to its target. As it dies, each helper gets
ActorControl 417 with no one (0xE0000000; Kottos' three too) and they go.

**28462's and 28760's effect** (what Kawari can't yet send): on whom they reach, two effect entries, `kk 00 1b 00 00 00
00 80` then the action ID (`2e 6f`, `58 70`), where the first byte is different for each target (0x42, 0x6B, 0x70, 0x82
in the runs) and isn't a known effect kind; no damage. Probably a knockback or a draw-in whose kind is the first byte;
not decoded (it needs the client's ActionEffect handler, or a sheet). The scripts send no effect.

**Death mid-cast.** A boss that dies casting: ActorControl 15 (CancelCast) with 540, 1, the action, 1, then the death
animation (the second boss, 28462). Kawari: `kill_actor` cancels an NPC's pending casts and sends it so.

The helpers are held (`director:hold_bnpc`): `npc_behavior` leaves them alone, so they don't notice, chase or follow a
timeline; they only do what the script has them do. Damage of 28544 and 28549 (Kottos', nobody caught) is a stand-in.

**Not here:** the helpers' SetTarget on the player, the shortcut, the end's log 9393 (24200), what 28462's and 28760's
effects are, statuses 8, 18, 53, 203, and ActorControl 15 (0, 1, 28764, 0) just before 28764 lands.

## The Thousand Maws of Toto-Rak

From Bob's Chronofoil captures of 2026-10-30: `caa5e908` visit 0.15 (27 minutes with Duty Support, during Into the
Beast's Maw, quest 514, which unlocks it; one death and a walk back in), `1fac2416` visit 0.15 (a Duty Finder run
with three other players, 10.6 minutes) and `2528a32d` visit 0.14 (another, 9.2 minutes, which checked everything
below: the same webs, objectives, adds, tail, eggs and spots), read with `questkit chronofoil --battle` and the visits' packets (numbers
only). The script's name, `f1r1_re`, is the ContentFinderCondition's ShortCode (row 1, checked against the sheet) and
the drop-in's LGB is `bg/ffxiv/fst_f1/rad/f1r1/level/planmap.lgb` (TerritoryType 1039's `Bg` folder; the LVB's own
list not checked: the server log says "Loaded dropin" if it matched).

**Enemies.** `questkit chronofoil --dropin 1039` over the three captures gave the 50 battle NPCs seen (level 24; trash 1340
HP). The bosses' adds and eggs (260, 271, 572: one layout ID per kind, spawned again and again) and the last boss (259)
are `nonpop`; the room bosses (270) stand from the start, untargetable (their display flags, 0x8, as retail's spawn).
The last boss's tail (266) had no layout ID on retail (it comes off the boss); Kawari spawns by layout ID, so it has
the fork's 99000266 in the drop-in. Every kind has a timeline from the two runs (`TotoRak*_<base>.json`): the casters
(1356, 1358) cast every 3.1 s instead of auto-attacking, 1353, 1357, 1359 and 1360 use a weaponskill every 4.4-7.8 s,
1355's 364 and 1360's 345 poison (status 18, 15 s), the room bosses' 1014 every 6.4 s round themselves poisons everyone
(status 2089, 15 s). Retail's ActionEffect for an action that only poisons gives the tick with the status (kind 14,
status, tick, 50: 57 on the player, 19 on a Duty Support member for 1014); such an action's `recorded_amounts` is now
its tick (`action.rs`, `set_npc_periodic_damage`; a DoT is otherwise scaled as its hit was, and these have none, so it stayed
what the script worked out). New action
scripts: 345, 346, 364, 697, 698, 701, 702, 704, 730, 1014 (344, 390, 968, 970 and 977 were there).

**The webs.** Six webs close the way (the director's map effects 0-5, set up as Kawari does every InstanceContent's);
each burns away (DirectorMapEffect, state 1, timeline 2) as the last of the pack guarding it falls, its health at 0,
whoever that was, in all three runs: 9074838-9074840, 9074846-9074849, 9074872-9074875, 9074881-9074883, 9074888-9074890,
9074898-9074900. The second and fourth open a room, and with them its boss (270) becomes targetable (ActorControl 54 in
the same tick, `set_targetable`). The web's collision isn't known to
Kawari's: party NPCs get past a web by walking on through a wall after 2 s, as they do any wall in a duty.

**The objectives.** Toto-Rak counts its progress with the duty's objectives (InstanceContent 1's six, InstanceContentText
33300-33305), not DirectorVars (sent once, all 0): ContentDirector events 0x80000016 `[done, 1]` and 0x80000015 `[next,
0]` (`director:objective`), the same in all three runs: entering the first room (0), its boss (1), entering the second (2),
its boss (3), entering the last (4), the last boss (5, with no next). A room counts as entered as the first player
reaches it, about its line: 23.3 and 23.7 yalms from the first boss (a position at 23.7 didn't, before), 22.1, 22.3
and 22.1 from the second (24.0 didn't), 41.0, 41.7 and 40.7 from the last (43.0, 43.6 and 43.1 didn't), whoever it was
(another player in both Duty Finder runs): `reach` 23.5, 22.5 and 42.0.

**The rooms** (`boss_room`: sealed 15 s after the boss's pull, log 2012 (PlaceName, 15) then 2013, the walls hidden at
the start as retail spawned them; opened at its death, log 2014):

| Boss | PlaceName | Wall / line / barrier (layout IDs) | Retail |
|---|---|---|---|
| 270 (3642541, 6248 HP) | 523 | 9089607 / 9089609 / 9087967 | The battle music (37) at its pull, off at its death; the barrier on the far side (EObj 2002872, standing from the start) hidden at its death. 1014 0.1-0.7 s after its pull, then every 6.4 s. Coffers 22 and 21 |
| 270 (3642543, 6248 HP) | 522 | 9089610 / 9089612 / 9087976 | The same, and two adds (271, both layout 3656529, 467 HP) at 50 % (49.8, 45.7, 48.2 %), always at the same two spots. Coffer 23 |
| 259 (3446839, 10451 HP) | 612 | 9089456 / 9089470 | See below. Coffer 27 |

**The last room.** Reaching it: objective 4 done, cutscene 95 (scene 3 with 95, 1, 38, 1, 0; flags 0x8502403),
the weather 19 (forced, 20 s fade) and the boss 2.5 and 2.8 s later; at its death the cutscene 173 (1, 18, 0, 0), the
weather 2 (4 s), coffer 27, the duty complete. The boss, from its pull (both runs to a few tenths): 697 (instant, on its
target and those in front) at 7.6-9.1 s then every 27.6; 701 (2.2 s, round itself) at 16.1-17.4 then every 36.6 (30.7
to 51); 698 (1.2 s) at 25.1-25.3 then every 27.8, on someone other than its target, and 4.7 s after it began an egg
(572) lands where they stood (`onActorCast`'s target position); 5.0 s later the egg casts 730 round itself (2.7 s: a
hit, status 2089) and it dies 2.1 s after that (`kill_bnpc`). Two adds (260, layout 3862492) at 75 % (74.7, 73.9, 74.5 %)
and two at 30 % (29.5, 28.6, 30.0 %), at the same four spots every time. At 55 % (it came at 54.2, 47.1 and 52.1 %:
retail waited for what the boss was doing, and in the third run its 701 ended just after the health fell past 55) 704 on itself, then 702 2.7 s later: as it begins the tail (266, 1278 HP) comes off where
the boss stands, the boss gets ActorControl 31 `[0, 0, 0, 1]` and 30 `[0, 1, the tail's actor ID, 0]` and log 2016
comes; while the tail stands 702 again every 30.8 s; the tail's death: ActorControl 20 `[64]`, 30 `[0, 2, the tail, 1]`
and 31 `[0, 0, 0, 0]` on the boss (`bnpc_control`). What 30 and 31 do isn't known (Kawari has no name for them).

**The slime** (`server/status_floor.rs`, `resources/data/status_floors.json`; Bob's bundle 20261030-171016: it gave
nothing). The slime at the last room's entrance (x about 199 to 207.5, z -137.5 to -144) and in patches on the boss's
platform (about (228, -150)) gives status 67 with param 30 while a player stands on it: ActorControl 20 (GainEffect)
`[67, 30, 0]` and a StatusEffectList with it (source 0xE0000000, no time limit) as they step on, ActorControl 21
(LoseEffect) `[67, 30, 0xE0000000]` and the list as they step off, within a quarter second of the position update
that took them there; a jump over it gave nothing until the landing. 17 times in the three runs (5 to the recording
player). It's a surface of its own, raised above the floor: on it, retail's players stood at y -38.98 to -39.13, beside
it on the platform at -39.29. So Kawari tells it by its collision material: each triangle of a zone's collision now
keeps its PCB polygon's material (`kawari-navimesh`, collision file version 3; version 2 files still read, without
them, and a zone with status floors has its navmesh made again), and the floor's materials are those under eight
points recorded on it, checked against eight beside it ("Status floor 67 (param 30) in 1039: materials [...]" in the
server log, or a warning that it was left out). A player in the air (no floor within half a yalm) keeps what they
had. Any duty can have such floors: an entry in `status_floors.json` with retail's points.

**The exit.** Toto-Rak's exit is EObj 2000683, not 2000139 (whose EObj Data, GimmickAccessor 5, it shares): retail spawned
it hidden and showed it (SetEventState 0) as the duty was complete. Kawari now hides every event object whose gimmick is
a duty exit (GimmickAccessor of the DutyExit kind, `Zone::exit_eobjs`) and shows them all with `complete_duty`
(`LuaDirectorTask::ShowExits`); it only knew 2000139.

**Coffers** (`treasure_loot`): 22 held 7035 or 31446; 21 2749 both times; 19 3584 both times; 29 3093 or 3095; 23
3093, 3095 or 31442; 25 2159 or 1696; 27 38470, 31443 and 1907, or 2117. 19, 29 and 25 stand from the start: each came
into view as the players came near (25 3.7, 4.0 and 10.1 s after the fifth web burned, so not the director's). No wiki
pools yet.

**Duty Support.** The trials' three: the Marauder (40), the Thaumaturge (42) and the Conjurer (43), at level 24 with
842, 583 and 588 HP (`party_npc::DUTY_LEVELS`; the Lancer, 41, wasn't there: 639 is a guess). Their story lines: the
tank's 14331 18.1 s in, the Conjurer's 14336 as the first room was entered, the tank's 14337 10.6 s after the last
room's cutscene began. Chatter: the Marauder's 13951, 13952 and 13953, the Thaumaturge's 13972 (now among their lines).

**Not here:** the shortcut (2000700, GimmickAccessor 74; shown by the time of the Duty Finder run's wipe, after the
second room; its pop range needs the game's layout files), what 30 and 31 on the last boss do, the event object retail
put near it 4.0 s into each 702 (EObj 2000404, the director's, not in the layout: hidden as the tail died, deleted 3.4 s
later; `spawn_eobj` can now spawn one the layout hasn't, since Haukke Manor, but this script doesn't yet), the item every player got as the duty was complete (log
789 and 1232 with it: 3095, 31443 and 2013 in the three runs), the webs' collision, status 9 (an auto-attack's, twice).

## Haukke Manor

From Bob's Chronofoil captures of 2026-11-02: `5d9c7d7f` visit 0.2 (48 minutes with Duty Support, during Skeletons in
Her Closet, quest 801, which unlocks it; one death and the shortcut back) and `e7580e0d` visits 0.1, 0.3 and 0.5 (Duty
Finder runs with three other players: twice as the tank, once as the healer, 9-12 minutes each), read with `questkit
chronofoil --battle` and the visits' packets (numbers only). The script's name, `f1d2_re`, is the ContentFinderCondition's
ShortCode by analogy with `f1d1_re` (not checked against the sheet: the server log says if `content/f1d2_re.lua` isn't
found); the drop-in's LGB is `bg/ffxiv/fst_f1/dun/f1d2/level/planmap.lgb` (Bob's dump of it, 2026-11-02, has the
event objects, pop ranges, event ranges and coffers named below).

**Enemies.** `questkit chronofoil --dropin 1040` over the two captures gave the 57 battle NPCs seen (level 28; trash
2420 HP, 276 1210). The pair (273, 274) and the last boss with its adds (14504, 14505, 14506) are `nonpop`: the
director spawns them; the first boss (272) and the floating eyes (275, 14698) stand from the start. Every kind has a
timeline from the four runs (`HaukkeManor*_<base>.json`): the casters (1368, 273) and the eye 275 cast every 3.1-3.2 s
instead of auto-attacking; the first boss casts 489 and then 855 every 18.4 s and 705 on a player every 50.8 s; the
last boss's rotation (`from_pull`) was the same to a few tenths of a second in all four runs. New action scripts: 378,
488, 489, 510, 514, 705, 855, 859, 860, 28640-28649, 29709, 29710, 29776 (`HaukkeManor*`).

**The keys and doors** (GimmickAccessor events, 0x000F00xx). Something to pick up has no prompt: the event starts, an
EventAction 21 runs 2 s, then log 2031 with the item, the object hidden (ActorControl 54 0 and SetEventState 7) and the
event finished. A door is a prompt (scene 1, flags 0x2001), then EventAction 24 for 2 s, log 2059 with a parameter of its
own, the door hidden the same way. Every run the same:

| EObj | What | Retail |
|---|---|---|
| 2000302-2000305 | the keys lying about (item 2000260) | each counts one more key held (director data 0) |
| 2000329, 2000337, 2000343, 2000344 | the plain doors | each uses up a key (log 2059 `[757]`) |
| 2000324 | the first boss's key (2000263) | dropped where it fell (spawned as it died, args2 1282): data 1 \|= 1 |
| 2000355 | the door it opens | log 2059 `[655]`; the key isn't used up |
| 2000325 | the cellar's key (2000264) | data 1 \|= 4 |
| 2000356 | the cellar's door | log 2059 `[756]`; the pair come out 0.4 s later |
| 2001235 | the pair's key (2000259) | dropped where the last of the two fell (spawned, args2 66818, not in the layout): data 1 \|= 2 |
| 2001233 | the stairs up | without the pair's key, its own log 2004 and the event refused (EventFinish result 0x65), no prompt; with it, the prompt, EventAction 24 and log 2023 `[0]` |
| 2012969 | the way back up from the cellar | shown as the pair fall; a prompt, then its own log 2049 (from GimmickAccessor 0x41A's handler) and the party at the entrance (pop range 9228207) |

The rest of the keys' series (2000306-2000308) and of the doors' (2000331, 2000339, 2000345), never used in the four
runs, are taken to work the same; 2000338 stood open (spawned in state 7, every run). The script shows every closed
door at the start (as retail spawned them, state 0), so they're the director's gimmicks: bots and enemies don't walk
through them before they open (Bob's bundle of 2026-11-02: the tank bot went up through the locked stairs). The director's variables
(DirectorVars, sequence 1 throughout) are those two numbers: 0, 0 at the start; 1, 0 with a key; 0, 0 through a door;
1, 1 with the first boss's key; 2, 1, 1, 1, 1, 5 and 1, 7 as the rest come. The script sends the same (checked with
lupa, the Duty Support run's steps).

**The first boss** (272, 17136 HP, standing in its hall; arena wall 2001605, line 2001604, PlaceName 604): its pull
seals the hall (log 2012 `[604, 15]`, 15 s later the wall and log 2013), starts the battle music (37) and shows the
shortcut. Its death: log 2069 `[423, 2000263]` (its BNpcName, the key), both wall and line hidden and 2001236 (the way
on, standing from the start) too, log 2014, the music off; coffers 73 by it and 70 and 71 in the cellar.

**The pair** (273, the caster, 7068 HP; 274, the fighter, 7768 HP; wall 2001607, line 2001606, PlaceName 609): out of
the cellar's door, both going for the party together (8 s later, as it came in); the room opens as the **last** of them
falls, whichever it is (273 in one run, 274 in three): log 2069 with that one's BNpcName (426 or 427) and the key,
2012969 shown, the walls hidden, log 2014, coffers 77 by them and 69 upstairs. Kawari: `boss_room` called for each of
them with the same wall now makes one room of them (sealed by either's pull, opened as the last dies:
`DirectorData::on_actor_death`).

**The last room.** Past the stairs, coming to its door plays the cutscene (scene 3: 170, 1, 38, 1, 0); the door (2000357,
closed from the start) opens with it, and the last boss (14504, 21063 HP) and 14505 (714 HP, untargetable by its
display flags) are there 2.4 s later. The boss's pull seals the room (log 2012 `[608, 15]`, then the wall 2000357 again
and 2013); 14505 joins in 15.4 s after the pull and 14506 (2142 HP) comes 34.0 s after it, the same to 0.1 s in all four
runs whatever the boss's health (88, 47, 43, 55 %). Its death: log 2014, the walls (2000357, line 2001080) hidden, the
exit shown, the clear rewards (log 9393 `[39000]`, the first clear's EXP, and 2163 `[4300, 3800]`, the first-timer
bonus), cutscene 171 (1, 18, 0, 0), 14505 gone, coffer 79. Which event range is the cutscene's rect isn't known: the
script takes any of the three by that door (3943563, 4105682, 9393269), once the stairs are open.

**The shortcut** (2000700): shown as the first boss is pulled, taking the party outside its hall (pop range 11438017;
retail's warps put them at (-41.2, 0, 12.5)), and from the pair's pull outside theirs (11438018, (-30.8, -18.8, 0.4)).
Outside the last room (11438019) from its cutscene isn't seen on retail. Kawari's shortcut gimmick now takes a pop range
of its own when its GimmickAccessor has one (Param1) and isn't the duty's shortcut, with its line (Param2) from its own
handler; if 2012969's isn't a shortcut, the script sends the party up itself (`director:warp_to_pop_range`).

**Coffers** (`treasure_loot`): 66 and 68 stand from the start; 73 held 31464, 31464 and 4122, or 31456 and 4112; 77
31459 and 4451, or 31451 and 4445; 79 9198 (the Duty Support run's, alone), 31466 and 2165, or 31460 and 1844; 68 4116.
70, 71 and 69 were never opened. No wiki pools yet.

**Duty Support.** The trials' three: the Marauder (40), the Thaumaturge (42) and the Conjurer (43), at level 28 with
1026, 710 and 717 HP (`party_npc::DUTY_LEVELS`; the Lancer, 41, wasn't there: 779 is a guess). Their story lines: the
Marauder's 14339 24.7 s in and 14345 as the first boss is pulled (retail: 3.9 s before, as the party came into its
hall), the Conjurer's 14344 32.8 s in and 14349 12.6 s after the last room's cutscene began, the Thaumaturge's 14347 as
the last of the pair fell. Chatter: the Marauder's 13951, 13952, 13953 and 13961 (now among its lines), the
Thaumaturge's 13972.

**Not here:** the stairs' locked log comes from the director after a prompt (retail: from the gimmick, with no prompt);
the exit's log 1534 `[6]`; what 28640, 28644 and 28649 (on the last boss itself, no effects) and the eyes' gazes do; what the
director events 0x40000007 (`[0, 1]` 0.2 s after the first door, `[1, 1]` 25 s after the first boss's key) and
0x80000004 are; wiki loot pools.

## The Bowl of Embers

From Bob's Chronofoil capture b8432613 of 2026-10-27, visit 0.16 (6.3 minutes with Duty Support, during Lord of the
Inferno, quest 343, which unlocks it; Bob a level 22 Conjurer): two attempts at Ifrit, the first a wipe with the
Infernal Nail standing, the second won. Read with `questkit chronofoil --battle 0.16` and the visit's packets (numbers
only), with Cactbot's ifrit-nm timeline (see [Sources](sources.md)) for the loops neither attempt finished. The
ContentFinderCondition's ShortCode (`w1fa_re`) and territory 1045's LGB (`bg/ffxiv/wil_w1/fld/w1fa/level/`) are
XIVAPI's (2026-10-27).

**Enemies** (drop-in `w1fa_re_planmap.json`, all `nonpop`: the director spawns them): Ifrit (BNpcBase 207, layout
4126276, level 20, 12885 HP) at (15, 0, 0) facing west; the Infernal Nail (208, 4126281, 1652 HP) in the middle,
spawned hidden and brought into view; his helpers (401, all layout 4126284, hidden; retail spawned twenty with him,
Kawari nine, the most that cast at once).

**Ifrit** (`IfritBowlOfEmbers_207.json`, `from_pull`; both attempts alike to a few tenths): his auto-attack 872 every
3.0 s; from the pull Incinerate (453, a 10-yalm cone, on the tank) at 7.2 s, Vulcan Burst (454, 16 yalms round him, a
knockback that did 1) at 15.0 s and every 41.6 s, Incinerate 7.9, 20.5 and 33.6 s after each. Below 80 % (79.9 and
79.5 % as he spoke) his line 2601 (battle talk under BNpcName 1185, no portrait, 5 s) and a new phase: Incinerate 1.2
s on, Eruption (455) 7.0, Vulcan Burst 13.6, then Incinerate and Eruption every 16.5 s from 20.5 and 26.3. Below 50 %
his line 2602 and the next: ActionTimeline 1398 (summoning) 4.0-4.3 s on, the Nail 7.8-8.2 s on (Appear 142 and
targetable 0.7 s later), Incinerate at 8.9, Vulcan Burst at 16.7, Incinerate at 24.6. The Nail's health at 0: status
775 on him (9.8 s), his line 2603, ActionTimeline 140; 2.6 s later ActorSetPos to the middle facing north and 141; 3.4
s after the Nail Hellfire (458, 1.7 s, 30 yalms round: 230 to the player, 166-183 to the Lancer and Thaumaturge, 74 to
the Marauder); 775 gone with his line 2604 9.8 s after the Nail; no auto-attack for 10.8 s. Then from Hellfire's cast
Incinerate 7.6 s on, Eruption 14.4, Radiant Plume (456) 23.0 and 31.4, every 32.2 s. At 0 his line 2605, the duty
complete (the director's 0x40000003, then 0x40000007 (0, 0) and (1, 0)) and the closing cutscene (scene 3: 87, 1, 18,
0, 0). The director begins each phase (`restart_timeline`), as his phases follow his lines and the Nail, not times.

**His helpers:** each Eruption (455, cast on himself, 1.9 s) is a helper's 733 (2.7 s, 8 yalms round, 185-227 to the
player) under someone of the party 0.1 s on: the Lancer, the Thaumaturge or the Marauder in all nine (never the player,
which may be chance; Kawari picks anyone of the party). Radiant Plume (456) is four helpers' 734 (2.7 s, 8 yalms round)
8 yalms from the middle (north, south, east, west), then the next one eight 20 yalms out (those and the diagonals),
in turn (181-227 to the player). Retail aimed them at the ground (TargetArea, the helpers never moved); Kawari puts a
helper there and has it cast on itself.

**Not killing the Nail:** the first attempt wiped 20 s after it appeared (the Marauder had fallen to Incinerate and
Ifrit turned on the player). Cactbot has Hellfire 44.3 s after the 50 % line whatever happens; Kawari does Hellfire
then too, the Nail gone (what retail's does to the party then isn't recorded).

**The Duty Support members' lines** (NpcYell): at the pull the Thaumaturge's 14326 (first attempt) or the Lancer's
14327 (the second), the Thaumaturge's 13693 0.3 s before the Nail appears (both), and her 13981 as Ifrit fell below 4 %;
the Marauder's chatter 13952. See [Party NPCs](party-npcs.md) for the members (40-42). With a player tank (Bob's
capture c99fe374 of 2026-10-28, visit 0.33: the Conjurer, 43, in the tank's place, a wipe then a win) the Conjurer said
14328 at the first pull and 14330 0.2 s before each Nail, the Thaumaturge neither, and nobody 13981; the Lancer's 14327
at the second pull as before (`content/w1fa_re.lua`: the Conjurer's lines when it's there, the Thaumaturge's otherwise).

**The wipe** (`server/wipe.rs`, any InstanceContent): every player down (the members were still standing) and 1.0 s
later the director's PartyWipe (0x40000005); 4.1 s after the fall 0x80000029, 0x40000013, the time left (0x4000000F,
3489), everyone sent back to the start alive (PrepareZoning to the same territory with warp type 25, `Event`, the
character hidden, flags 2; ActorSetPos with the same warp type; SetMode normal), 0x40000011, the duty's enemies and
the party's NPCs spawned again where they began and the entrance circle shown again; 11.3 s after the fall
DutyRecommence (0x40000006: the time left, 0, 2) and the entrance circle hidden. Kawari sends the same, gets the
party's NPCs up beside their player 6.1 s after the fall (once the player has arrived), drops what the script had
queued and calls its `onPartyWipe` (Ifrit, the Nail and the helpers taken away and Ifrit brought back) and
`onDutyRecommence`. Before, nothing happened until the player chose to return, and the fight went on without them.


**Fixes from Bob's bundles of 2026-10-28** (his first runs; untested since):
* *Facing at the entrance:* the InstanceContent has no entrance rect and the entrance circle faces north, so Kawari
  turned players north; retail turned them to 1.571, toward Ifrit, both times. A duty script's `ENTRANCE_ROTATION`
  now says, where there's no rect (`change_zone_warp_to_entrance`), for coming in and after a wipe.
* *The next attempt after a wipe:* the old Ifrit's health was still there for an update after `onPartyWipe`, so his
  80 % and 50 % lines and the Nail came again 8 s into the next attempt; the script now waits for the new Ifrit at
  full health. The members got up at a fixed 6.1 s, before the player's client had said where it arrived, so they
  stood up where the player had fallen, in the arena; they now get up once the player's position has moved from where
  they lay (or at the recommencement).
* *Too much damage:* Ifrit's hits on the player matched retail (Incinerate 163 against 166, Eruption 219, Radiant
  Plume 208, Hellfire 224), but the members took far more: they stood in every Eruption and Plume (the helpers aren't
  hostile, and only hostile casters' areas were dodged; now a director's held helpers' are too), the Lancer stood in
  front of Ifrit and took 15 Incinerates where retail's took 2 (only after the tank fell; melee bots now stand behind
  what the tank holds, `Placement::behind`), and everyone stood inside him (their fighting distance was from his
  middle; it's from his edge now, as retail's 6.5, 7.6 and 13-15 yalms against his 3.5 hitbox). So the party fell
  and the player with it.
* *Leaving:* retail sent the player on to Camp Drybone (145, ZoneInit pop range 4156164, the quest's POPRANGE1, at
  (-386.72, -59.0, 133.89)) during Lord of the Inferno, not back to Southern Thanalan: the quest's `onDutyCleared`
  sets it (`set_duty_exit`), so only for that step of the story.