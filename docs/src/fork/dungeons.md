# Dungeons

The duties the fork has scripted from retail captures: their director script (`resources/scripts/content/<short
name>.lua`, the ContentFinderCondition's ShortCode), their enemies (a drop-in, `resources/dropins/<short
name>_planmap.json`: the client's layouts have no battle NPCs), the enemies' timelines (`resources/timelines/`) and
action scripts. The director API they use is described in [Quest Battles and Directors](quest-battles.md); the Duty
Support members in [Party NPCs](party-npcs.md); coffers in [Coffers and Loot](loot.md).

| Duty | ContentFinderCondition | Territory | Script | Recorded |
|---|---|---|---|---|
| Sastasha | 4 | 1036 | `content/s1d1_re.lua` (upstream's, with the fork's changes) | five Duty Finder runs (2026-10-22), one with Duty Support (2026-10-23) |
| The Tam-Tara Deepcroft | 2 | 1037 | `content/f1d1_re.lua` | one run with Duty Support and two Duty Finder runs (2026-10-24) |
| The Copperbell Mines | 3 | 1038 | `content/w1d1_re.lua` | one run with Duty Support and two Duty Finder runs (2026-10-25) |

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

**The gimmicks** (GimmickAccessor events, 0x000F00xx; each a 2 s interaction, then the event finishes):

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
14312 26.5 s after the last seal is broken. The Marauder's chatter included 13953, now among its lines.

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

**The way through.** Retail's DirectorVars (sequence 1 throughout): data 1 is the progress (1 past the first door, 3
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

The refusals' logs are the gimmicks' own (`isGimmickFunctional`). The first lever was scripted as 2000161, which nobody can use: saying yes to its prompt did nothing and the run was stuck there (Bob's bundle of 2026-10-25); the EObj each gimmick event came from in all three runs is the one in this table. The EObjs hidden at the start (state 7 in their
first spawn) are hidden in `onSetup`. **Ambushes:** the two bigger gigas (1398 at layout 4023421, 1403 at 4023560) wait
in a pile of rock (2000168, 2000185); someone coming within 19.4-22 yalms (all three runs) hides the pile and the gigas
comes at them at once (it spawned and targeted the player in the same packet burst; ActorControl 36 (1, 158) and 54
(1) on it, not sent here). **The last hall:** the cutscene (scene 3 with 178, 1, 38, 1, 0) began as the first player
came down past z 55 (52-56 in two runs; players waiting at 58-63 didn't start it), the boss 2.4 s later, and
ActorControl 409 (64, the director) on EObj 4217260; there was no gimmick event.

**The bosses** (`spawn_boss`: sealed 15 s after their pull, log 2012 (PlaceName, 15) then 2013; opened at their death,
log 2014; the battle music, 37, at the first two's pulls and off at their deaths):

| Boss | PlaceName | Wall / line (layout IDs) | Retail |
|---|---|---|---|
| 14460 (9053767, 9970 HP) | 679 | 9085524 / 9085538, and 9085522 hidden at its death | 28545 (4.7 s) at 2 s and every 81 s; 28543 (7.7 s, hitting nobody, with its three helpers' 28544) then 28549 twice, 28546 (3.7 s) and 28547 (4.7 s) every 37 s, the same to a tenth of a second in all three runs. Coffer 52 |
| 14448 (9040077, 11383 HP) | 680 | 9089361 / 9089385, and 9089366 | 28462 (3.7 s) at 3 s, 28463 on itself (2.7 s) at 10 s and every 16.5-18 s (29.4 with Duty Support), each followed 8.6 s later by its six adds' 28465 (5.7 s, 121-127); 28461 once. Coffers 53 and 59 |
| 14537 (9060676, 10582 HP) | 681 | 9082207 / 9082214 | 28762, 28760, 28764, 28763 in a 47.7 s round from 5 s; its five helpers' 28761 (127) at 22 s and every 47.7 s. Cutscene 179 (1, 18, 0, 0), coffer 51, the duty complete |

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
after the last boss's cutscene begins. Chatter: 13962, 13964, 13972, 13974, 13953, all already among their lines.

**Not here:** the helpers (9020: three for the first boss, five for the last, all with one layout ID each, untargetable
lvl 1) and five of the second boss's six adds (one layout ID, 9040086, so one spot): one add is spawned with the boss,
its 28465 every 19.5 s from 19 s (the adds' cycle on retail followed the boss's 28463). Retail's adds showed and hid
(ActorControl 239) each cycle; the one here stays. The shortcut, the ambushers' ActorControl 36/54, the end's log 9393
(24200), what 28462 and 28760's effects (46/112, 88/130) and statuses 8, 18, 53, 203 do.
