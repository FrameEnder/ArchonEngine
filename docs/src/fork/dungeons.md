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
