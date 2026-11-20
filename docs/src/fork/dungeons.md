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
| The Navel (Titan) | 57 | 1046 | `content/s1fa_re.lua` | one run with Duty Support (650dbfd8) and two Duty Finder runs (0fd8bce0, 48aaf6fe), 2026-11-06 |
| The Howling Eye (Garuda) | 58 | 1047 | `content/r1fa_re.lua` | one run with Duty Support (6a5244ec) and two Duty Finder runs (09ecf8e7, 6c495fc4), 2026-11-08 |
| Brayflox's Longstop | 8 | 1041 | `content/s1d2_re.lua` | one run with Duty Support (44df02fc) and one Duty Finder run (c9145fdc), 2026-11-03 |
| The Stone Vigil | 11 | 1042 | `content/r1d1_re.lua` | one run with Duty Support (1cefedf4) and one Duty Finder run (341808b0), 2026-11-07 |
| Castrum Meridianum | 15 | 1043 | `content/w1d4_re.lua` | five runs with Duty Support (the fourth Bob's 6320ae56 of 2026-11-18, a Paladin; the fifth his deea324c of 2026-11-19, a White Mage, the fights drawn out): Connor's af05916c (2026-11-09), Bob's 9ba7d981 and 50e72631 (2026-11-17) |
| The Porta Decumana | 830 | 1048 | `content/w1d5_battle.lua` | four runs with Duty Support (the fourth Bob's 6320ae56 of 2026-11-18, a Paladin): Connor's af05916c (2026-11-09), Bob's 9ba7d981 and 50e72631 (2026-11-17) |
| The Praetorium | 16 | 1044 | `content/w1d5_re.lua` | five runs with Duty Support (Connor's af05916c, Bob's 9ba7d981 twice, 50e72631 and 6320ae56): the two lifts and three more objects, six cutscenes, the ride on the armour (Mount 17, action 1128), three bosses with their helpers, the last one's second part with its four and the gauge. No layout dump. **Unplayed** ([below](#the-praetorium)) |

Every duty script in this table is checked against AutoDuty's route for its territory (`questkit autoduty`, see
[questkit](questkit.md)): on 2026-11-13 each names every object AutoDuty uses, and Sastasha's keys are the one
difference (see `known-issues.md`). Brayflox's Longstop and the Stone Vigil joined the table that day; they were
scripted already.

## The duty's beginning: everyone waits for the openings

Retail begins a duty for everyone at once (DutyCommence to every player together) once each has finished its opening;
until then the players stand inside the entrance circle, which keeps them in, and nothing fights. Retail's client
also hears each party member's cutscene state (ActorControl 519 with the member and 3 watching, 1 done; their status
icon, 504, Viewing Cutscene meanwhile; Bob's Chronofoil capture 09ecf8e7, the Howling Eye). Kawari sends it since
2026-11-11, for every cutscene, not only openings (see [Quests](quests.md), "Watching a cutscene").

Kawari (since 2026-11-11; Bob's bundle 20261111-112150, "the battle was playing out while I was in a cutscene"):
an InstanceContent instance starts `awaiting_commence`; each player's return from the opening (`ReadyToCommence`) is
noted (`director::ready_to_begin`), and when the last player in it is ready the duty begins for all of them:
DutyCommence, the entrance circle hidden, the time left. Until then its enemies notice no one and do nothing (the
NPC tick passes them by), the party's bots stand where they came in and do nothing (`party_bot::update`), and the
director's `onUpdate` waits (its timers count from the beginning). Before, the first player's return began it alone,
and nothing held anyone back: in Bob's run the Duty Finder's bots and Garuda fought through his opening, and the
opening took 3 minutes to end (the client didn't leave it while he was in combat, it seems: in earlier runs it ended
3-18 s after the cutscene). Someone coming in after the beginning gets it alone. Quest battles begin by their scripts.
The Howling Eye's entrance circle (EObj 2000274, layout 4247837; ActorControl 106 with 7 at DutyCommence in capture
09ecf8e7) wasn't known, so its circle never hid, and nothing after the missing circle was sent (the time left) — it's
in `ENTRANCE_CIRCLE_IDS` now, and a duty whose circle isn't known gets the rest all the same. The duty's clock never
started either, and a wipe is only watched while it runs: that's why Bob's death there didn't reset the duty ("I died
and it did not reset").
Test `a_duty_begins_when_everyone_has_seen_its_opening`.

**The duty's time flags.** Retail's duty director sends its event 0x40000007 `[0, 1]` 300.0 s after DutyCommence and
`[1, 1]` 900.0 s after it, whatever the party is doing (every one of 65 and 20 runs that lasted that long: dungeons,
trials and raids in Bob's Chronofoil captures), and `[0, 0]` and `[1, 0]` with DutyCompleted (78 clears), set or not.
An unrestricted party gets `[1, 1]` with DutyCommence instead (Bob at 87 in the Binding Coil's turns and two level 50
Extreme trials, 32 runs). Quest battles' own directors never got one. What the client does with them isn't known (the
duty's time left is re-sent at the same 300 s, `SetDutyTimeRemaining`). Kawari sends them since 2026-11-11
(`director::DUTY_FLAG_TIMES`, checked on each tick with the time left in `resync_duty_clock`; `Instance::duty_flags`;
someone coming in late gets those already set). Test `a_duty_s_time_flags_come_when_retail_s_did`.

**The duty's music** is the opening scene's first parameter (EventScene, director 0x0C53A401): retail sends no BGM
director event (0x80000001) in the Howling Eye, and its opening carries 155 there in all three runs (09ecf8e7, 6a5244ec,
6c495fc4), 157 in the Navel (650dbfd8) and 0 in the Stone Vigil (1cefedf4), where the zone's own music plays until a
boss's 0x80000001. Kawari sent 0 (Sapphire's guess, "BGM?"), so Garuda fought to the zone's music ("what about the boss
music?"). Since 2026-11-11 it's the duty's InstanceContent BGM (`GameData::find_content_bgm`, passed with
`PlayInitialCutscene` from `director.rs`'s EnterTerritoryEvent). That InstanceContent's BGM is the number retail sends
is a reading of the column's name; there's no game install here to check that the Howling Eye's row says 155, so a
duty whose music is wrong in game is the first sign it isn't.

## Where players come in

Players come in at the duty's entrance circle: an EObj bound to the spot they stand on (a SharedGroup), which retail's
server hides with ActorControl 106 [7] at DutyCommence and shows again after a wipe. Kawari finds it by its EObj ID
(`ENTRANCE_CIRCLE_IDS`, core `common/game.rs`); a duty whose circle isn't there comes in at a random pop range of the
whole duty, its circle never hides, and the server log says so ("Territory …'s entrance circle isn't in
ENTRANCE_CIRCLE_IDS"). That happened in the Stone Vigil (2026-11-07) and again in Castrum Meridianum (Bob's mark of
2026-11-19 at Rock the Castrum: "the starting spawn point isn't even slightly set-up"; he came in at (-23.9, 70.1,
21.9) where retail put players at (247.663, 87.15, -94.968) in all six entries), which had five retail runs in hand:
nothing compared the captures with the list. The Praetorium's was missing too.

So `questkit chronofoil <captures> --layers` (run on every capture as it comes in) now also reads every duty entry
(`chronofoil_entrances.py`): where retail put the player, the client's first facing, and the circle (the first object
with a bound layout retail hid with ActorControl 106 [7], within 15 yalms of the entry point). It fails where
`ENTRANCE_CIRCLE_IDS` hasn't the circle or `entrance_facing.json` the facing; `--save` keeps the entries in
`questkit/data/entrances.json` (119 entries from the 14 captures in hand that entered a duty, 2026-11-19), which the tests check on every push
(`DutyEntrances`). The circles retail showed:

| Territory | Duty | Circle (EObj) | Layout | Where retail put players |
|---|---|---|---|---|
| 281, 1067 | guildhests | 2000182 | 4600249, 4377979 | (0.598, 0, 14.6), (0.971, 0, 14.278) |
| 1042 | The Stone Vigil | 2001869 | 4239683 | (0, 4.998, 306) |
| 1043 | Castrum Meridianum | 2000562 | 9088360 | (247.663, 87.15, -94.968) |
| 1044 | The Praetorium | 2000790 | 9058469 | (149.517, 187.039, 0.441) |
| 1047 | The Howling Eye | 2000274 | 4247837 | (-0.113, -2.029, 17.533) |
| 1048 | The Porta Decumana | 2007457 | 9088353 | (-758, -400, -600) |

The other duty entries in the captures (quest battles, trials, the Hall of the Novice) showed no circle.

**By layout ID, not EObj ID** (2026-11-19, Bob's next run: "stuck in circular barrier"): an EObj ID isn't one duty's
circle alone. 2007457 is the Porta Decumana's circle and also Castrum Meridianum's way to its last hall (layout
9090412), and it came first in the list: at DutyCommence (16.9 s in) Kawari hid that, and the real circle (2000562)
never hid, so he couldn't leave it. The circles are now recorded by territory and layout ID,
`resources/data/entrance_circles.json` (`entrance_facing.rs`'s `entrance_circle`), which `Zone::find_entrance` and
`Instance::find_entrance_circle` read first; the EObj IDs are only for a duty not recorded, and the `--layers` check
compares layout IDs (test `the_entrance_circle_is_retails_by_layout`). His entry itself was retail's (247.663,
87.14987, -94.968) on `95fe982`. Kawari puts players at the circle's bound object, which it reads from the layout; that the spot is retail's isn't checked
for Castrum Meridianum and the Praetorium (no layout of theirs here).

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
2026-11-02; the other three's first updates came 8.7-22 s in); Brayflox's Longstop (1041) is -1.937, the first position update of Bob's capture 44df02fc (2026-11-03, 1.3 s in); the Stone Vigil (1042) -3.1416, the first position update of all three of Bob's runs of 2026-11-07 (its entrance rect faces 0). What decides the facing, in order (`change_zone_warp_to_entrance`, coming in and
after a wipe): a duty script's `ENTRANCE_ROTATION`, the recorded facing (16 more territories since 2026-11-19, from
the duty entries `--layers` reads: 262, 281, 304, 305, 308, 309, 313, 320, 326, 330, 1047, 1049, 1052, 1127-1129; and
1043, 1044, 1048), the entrance rect's, the entrance object's.
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

**Enemies' gil**: in a duty with another player there (or a party bot, standing in for one: since 2026-11-10, Bob's
roulette run of the Brayflox's Longstop with three bots dropped nothing, bundle 20261110-111747), each of the zone's own enemies (spawned as the instance was
made, `Instance::layout_npcs`; not the bosses, adds and waves the director spawns) drops gil for each player it
credits: its level × 0.400-0.532, rounded down (`actor::kill_gil`), after the kill's EXP, as the gil's slot,
LootMessage 9 with the amount and ActorControlSelf 42. Retail: 133 of 133 such kills in the Duty Finder runs dropped
(level 15: 6-7 gil, 16: 6-8, 17: 6-9), none of the 83 boss, add and wave kills, and none of the 208 kills in Duty
Support runs (no other player) ([Coffers and Loot](loot.md#enemies-gil)).

## Sastasha

`content/s1d1_re.lua` (upstream's, with the fork's changes). What eight retail runs say of its bosses (Bob's
Chronofoil captures 700cc294, c99fe374, cc62ad94 and f4316611's five; `questkit duty-sim 4`), in the script since
2026-11-10:

* **The music** (director 0x80000001): 37 as Chopper appears, and as each captain is *pulled* (17-87 s after he
  appeared; the script started it as he appeared); 0 as Chopper falls and as each captain leaves. None for Denn, and
  none stopped as he falls.
* **Neither captain dies.** The first gives up at half his health (the last hit before had him at 48.2-50.2 %) and
  runs down the way on, gone 21.5-22.5 s later at (-128.5, 17.2, 155.6); the second at a fifth (18.1-20.8 %), gone
  22.1-25.9 s later about (-315, 5.9, 306) (15.8 s in one run, further back). Kawari: `set_invulnerable`,
  `set_notice` off, `retreat_bnpc` to the spot, `leave_boss_room` (his room opens), `despawn_bnpc` 22 and 24 s later
  (`captainLeaves`); what followed his death follows his leaving (coffers 95 and 100, the next part, the members'
  lines). Before, he fought to 0 and fell. What retail sends as he turns (a line, his being untargetable) wasn't
  read, nor when the coffers come.
* **Denn appears as the party reaches his cove**: first seen 50-77 yalms from the capturing player, 27-226 s after
  the second captain left and 6-29 s before his pull. Kawari spawns him with the cove's cutscene (`onGimmickRect`,
  once the captain has gone); before, as the captain fell.
* **At a wipe** (`onPartyWipe`; none on retail here): the music starts again at the next pull of a boss still there;
  Denn's waves are taken away and counted again from his next pull.

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
2. The pack's last death calls the room's boss (567, 4128 HP): log 2009 (its BNpcName 455, 1) and it goes for the
   party at once. Fifteen rooms in five runs (3ec702a8, c99fe374, cc26e0ea's two, cc62ad94): it appeared 0.1-0.5 s
   after the last of the pack fell in twelve, whichever that was (the leader had fallen up to 7.2 s before); in three
   it came with one of the pack still standing at 1-5 % of its health, 1.5-1.8 s before that one fell (what decides
   those isn't known). Until 2026-11-10 the script called it as the leader fell, so its music stopped 3-7 s early
   (`questkit duty-sim 2`).
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
28544's damage is a stand-in (28546's, 44.5) since nobody was caught by either (`Helper_9020.json`).

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

**Its invisible walls.** The edge of the sunken space by the west wing (z 45, x -32 to -21) is closed by BgParts whose
collision is a simple shape (3819851, 3819852: boxes 2.4 yalms wide, 5.4 high; 3867912: 3.1 wide, 3.9 high), from
Bob's `bg.lgb` (2026-11-02). Kawari's collision has them since collision version 4; before, the tank bot and three
enemies walked off that edge to the cellar ([Party bots](party-bots.md)).

**Its doors.** The layout places one door shared group (`sgbg_f1d2_a1_door1.sgb`, `_door2`, `_door4`; the cellar's
`sgbg_f1d2_b1_door*.sgb`) in each doorway, most of them open (`BG_a1_open_door`: the hall's east doors at x ±49, z ±36;
x 16.5, z 69.3 and 84.6; x -23.3, z 0; x -16.5, z 69.6) and some shut for good (`BG_a1_close_door`). Those an event
object controls (the keys' doors, the bosses') follow the director. Kawari had every one shut; since collision version 5
the open ones are open ([Party bots](party-bots.md)).

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
own, the door hidden the same way. Every run the same. Kawari ends the bar with ActorControlTarget EventAction `[0, id]`
on the object since 2026-11-07 (`QueuedTaskData::CastEventAction`), as retail does for a quest battle's (In the Eyes of
Gods and Men's rosary) and as the quests' bars always were:

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
| 2012969 | the way back up from the cellar | shown as the pair fall; a prompt (yes answers 0, no 1), then its own log 2049 (from GimmickAccessor 0x41A's handler) and the party at the entrance (pop range 9228207); no does nothing (Kawari warped either way until 2026-11-03: Bob's bundle 20261103-130104) |

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

**The eye at the last room's door** (275, 8568 HP, layout 3633689, in the corridor before it): it never dies. Below
60 % (retail's four runs: 59.9, 58.1, 57.5 and 57.9 % by the hit that crossed it) it gives up the fight: ToggleWeapon off
and SetBattle 0 from it, SetBattle 0 to the party, an unknown packet (opcode 942, `[the eye, 1]`), its cast cancelled
(CancelCast `[537, 1, 514, 0]` in the run it was casting), its target dropped, its health back to full over the next
second; it runs (speed 60) along the corridor to x 24.5-25.4, z 0 (1.7-2.6 yalms short of the door, at 22.8), 1.8 to 8 s
depending on where it was. As it gets there the door (2000357) opens (SetEventState 7) and the director's cutscene
starts at once (scene 3: 170, 1, 38, 1, 0); it drifts on through the doorway at a walk (speed 20) and is deleted 1.1-1.4
s later. Kawari (`f1d2_re`, since 2026-11-03): below 60 % `set_invulnerable`, `set_notice` off and `retreat_bnpc` to
(24.5, 17.07, 0); within a yalm of there (`onUpdate`'s `info`) it's held, walks on to x 23.3 (`move_bnpc`, speed 20),
the door opens with the cutscene, and it's despawned 1.3 s later. Killed some other way, the door opens all the same.
Before, the script waited for a player to come into an event range by the door, which they couldn't reach with it shut
(Bob's bundle 20261103-121655: the eye killed, the door never opened).

**The last room.** The door opening with the eye plays the cutscene (above); the last boss (14504, 21063 HP) and 14505
(714 HP, untargetable by its display flags) are there 2.4 s later. The boss's pull seals the room (log 2012 `[608, 15]`,
then the wall 2000357 again and 2013); 14505 joins in 15.4 s after the pull and 14506 (2142 HP) comes 34.0 s after it,
the same to 0.1 s in all four runs whatever the boss's health (88, 47, 43, 55 %). Its death: log 2014, the walls
(2000357, line 2001080) hidden, the exit shown, the clear rewards (log 9393 `[39000]`, the first clear's EXP, and 2163
`[4300, 3800]`, the first-timer bonus), cutscene 171 (1, 18, 0, 0), 14505 gone, coffer 79.

**The last boss's fight** (since 2026-11-03; Bob's Chronofoil captures, the four runs of 2026-11-02 and 25342d59 of
2026-11-03, the same to a tenth of a second whenever it got that far): its timeline (`HaukkeManorFinalBoss_14504.json`)
has each action at its cast's start, every 63.1 s from 37.2 s: 28646 on a player, 28640, 28649, then 28644 and 28643 as
28649 lands, 28645, 28644 and 28640 (until 2026-11-03 the times were when each landed, so every cast came about 4 s late
and 28644 and 28643 came before the 28649 they follow; the first boss's 855 and 705 too, 3 and 4 s). The director
(`f1d2_re`'s `onActorCast`) plays the rest:

| Retail | Kawari |
|---|---|
| 28640 (4.0 s cast, on itself): 5.1 s after its cast began, the eye (14505) is put by the room's middle (-2.5, 17, 4: ActorSetPos, then ActionTimeline 7747 on it); 5.2 s later the eye casts its gaze (28648, 4.7 s bar) and 8.1 s after that plays 7737 | `warp_bnpc`, `play_action_timeline`, `actor_use_action` on the eye, held (`hold_bnpc`: it neither moves nor acts on its own; its timeline has no rotation now, it had a gaze every 34.2 s) |
| Its second 28640: 14506 comes 5.3 s on, unseen, and shows 0.85 s later (ActorControl 36 with ActionTimeline 158, 54) | `spawn_bnpc`, `appear_bnpc(…, 158, 0.85)`, `set_hate` (it was a fixed 34 s after the pull, and never shown) |
| From the third 28640 they take turns: the eye by a wall, at (-4.5, 17, 22) or (-4.5, 17, -14) (three runs and two), its gaze 7.2 s after, then by the middle again | the same, the wall at random |
| 28644 (instant, no target): the first as it's pulled; then each one leaps it (SnapNextMove, ActorMove speed 90) 0.8 s on beside the eye by the wall (x -0.5, the eye's z), where it stands 4.9 s, or 0.7 s on back to the middle (-7.52, 17.26, 3.98) | `jump_bnpc`; `root_bnpc` (new) for the 4.9 s |
| 28643 (instant, everyone in 100 yalms) 1.4 s after that 28644: the two Seduced statuses, and each player walked to it for 3.0 s, while the eye's gaze is cast | `effects:walk_to_caster(3.0, 4, 1132)` ([Combat: walked to the caster](combat.md)); the gaze lands 1.7 s after the walk ends, on those still facing the eye |

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
the exit's log 1534 `[6]`; what 28640, 28644 and 28649 (on the last boss itself, no effects) and the eyes' gazes do; wiki loot
pools. (The director events seen there are the time left, 0x80000004, re-sent every 300 s, and the duty's time flags,
0x40000007, both sent: above.)

## Brayflox's Longstop

From Bob's Chronofoil captures of 2026-11-03: `44df02fc` visit 0.20 (34 minutes with Duty Support, during The Things
We Do for Cheese, quest 832, which unlocks it at its sequence 3) and `c9145fdc` visit 0.14 (14 minutes, the Duty Finder
with three other players), read with `questkit chronofoil --battle` and the visits' packets (numbers only).
ContentFinderCondition 8 (ShortCode `s1d2_re`, from the sheet), territory 1041 (`ffxiv/sea_s1/dun/s1d2`), InstanceContent
8; the drop-in's LGB is `bg/ffxiv/sea_s1/dun/s1d2/level/planmap.lgb`. Bob dumped it and `bg.lgb` on 2026-11-03 (read
with physis' `Lgb`, numbers only): the event ranges, pop ranges and SharedGroups below are its. Retail's sequence was
1 throughout; both runs' DirectorVars were the same.

**Enemies.** `questkit chronofoil --dropin 1041` gave the 57 battle NPCs seen (level 32; trash 2258 HP). What the
director brings (new actor IDs on retail) is `nonpop`: the first boss (423) and its adds (424, 425, 426), the enemy and
goblin after it (1046 at 3992769, 420), the two ambushes (1044s 3977699-3977701, 3977704-3977705), the second boss
(990), its add (991) and the goblin who fights it (422), the third boss (427), its blobs (1064) and the dragon (429), the
last boss (14533) and its fourteen helpers (BNpcBase 9020, no layout ID on retail: the fork's 99009020), and three
goblins after the clear (4107001, 4107000, 4106998, from the Duty Finder run). Event NPCs: the key's (ENpcResident
1004346, layout 4000399) from the start, three (1043371, 1043373, 1043374) on the terrace after a Duty Support clear.
The goblins (420, 421, 422) carry retail's character flag 0x2 but are BNpcBase battalion 0, on the party's side. Every
kind fought has a timeline (`Brayflox*_<base>.json`: `players` from the Duty Finder run's four, `npcs` from the Duty
Support members); new action scripts: 322, 365, 448, 498, 505, 506, 508, 519, 564, 920, 979-982, 984, 985, 990-992,
1010, 1393, 28656-28660, 29617, 29618 (`Brayflox*`). 982 gives status 60 and no damage; 505, 506, 508, 448, 498, 29618 and
28660 caught nobody in either run.

**The way through** (EObj IDs; doors are GimmickAccessor events 0x000F00A2-A6). Retail's DirectorVars: data 0 the key
held, data 2 and 3 the progress (1 the key, 2 the first boss, 4 the second, 8 the terrace door; data 3 alone 16 at the
clear): 1, 0, 1, 1; 0, 0, 1, 1; 3, 3; 7, 7; 15, 15; 15, 31. The script sends the same (checked with lupa). Where the
runs differ, both are given.

| What | Retail |
|---|---|
| The dragon by the entrance (989, untargetable) | ActionTimeline 1385 3.7 s after DutyCommence, gone 10.3 s later |
| The key's event NPC (ContentNpc 1, handler 0x190001) | scene 4 (flags 0x2000); on its return log 2031 `[2000521]`, EventFinish, the variables; at once (0.4 s, 0) the first boss and three 425s in the pond. Another player's talk reaches everyone as the log and variables |
| 2001462, the pond's door | a prompt (scene 1, 0x2001), EventAction 24 for 2 s, log 2059 `[759]`, state 7, map effect 7 (state 1, timeline 2) |
| The first boss (423; wall 2001584, line 2001585, PlaceName 687) | log 2012 `[687, 15]` as it and its adds were pulled, the shortcut (2000700, by the entrance at (113.3, -1.2, 59.1)) shown, BGM 37; 2013 15 s later. Its waves: log 2052 and three adds 60.1 s after the pull (both runs), then every 55 s, each going for the party at once; the kinds a new mix each time (425, 425, 426; 426, 425, 425; 425, 424, 426 twice), the first wave at the same three spots in both runs |
| Its death | log 2014, 2059 `[760]`, 2001463 opened, map effect 8, BGM 0, coffer 109; an enemy (1046) and a goblin (420) running at it (143 twice, about 5 s later; 148 or 149 and ActionTimeline 957 3.6-4.1 s after the enemy fell, which drops coffer 105); the tank's line 14354 8.4 s later; the second boss 13.4 and 14.1 s later; the goblin on the ledge (4155136) calls 145 every 15 s from 18.9 and 23.0 s on |
| The ambushes | three 1044s by the goblin 4000414 as a player entered the event range 4000437 (both runs: 0.15 and 0.42 s after the first position update inside it), two by 4000410 as one entered 4000429 (0.4 s after; the Duty Support run only: the other party never went that way, and they never came), going for the goblins (145 from each; 146 15.8 s later from the first once); the first pack cleared drops coffer 106 (the second in one run, the first in the other) and its goblin cheers 3.3-3.5 s after (148 or 149, 957) |
| The second boss (990; wall 2001586, line 2001587, PlaceName 688) | at 80 % (both runs) its add (991) and a goblin (422, level 20, the party's side), fighting each other; boss and add hold the room together: it opened as the last of them fell (the boss in one run, the add 12 s after the boss in the other). The boss's 984 only after the add fell (3.7 s after, every 30.8 s). The goblin's lines: 150 18 s in, 151 with its 991 and 156 4 s on, 155 with its 990 and 157 6 s on, 158 5.0-6.3 s after the room opened |
| The room's opening | log 2014, 2059 `[761]`, 2001464 opened, map effect 9, coffer 110, the healer's 14357, the third boss spawned |
| The third boss (427; wall 2001588, line 2001589, PlaceName 689) | a 1064 2.6-2.9 s after each 979 where its target stood; at 20 % (both runs) the dragon (429), spawned unseen, Appear 0.6-0.8 s later with 564 on everyone, log 2055 at 4.6-4.8 s, 29618 at 13.1-13.3 s; it leaves as its health falls below 71 % (at 70.7 % 24.2 s in, and between 71.6 and 67.2 % 15.9 s in): untargetable, ActionTimeline 1385 1.4-2.3 s later, log 2056 7.0 s later, deleted 12.1-12.2 s later; the room opened (log 2014, 2059 `[762]`, 2001465, map effect 10, coffer 111, the healer's 14359) as it was deleted, the boss having fallen 4.6-4.9 s after the dragon came: **the dragon kills it**, Kill and SetMode dead with no hit, at 18 % in capture 44df02fc (`BOSS3_CRUSHED_AFTER`, 4.85 s after the spawn, since 2026-11-10; Kawari's lived on until the party wore it down, Bob's bundle 20261110-111747). 2001465 refused a player before that (EventFinish 0x65) |
| 2001466, the terrace door | no prompt, EventAction 24 for 2 s, log 2059 `[691]`, state 7, the variables 15, 15 |
| The last boss's cutscene | scene 3 (184, 1, 38, 1, 0) as a player entered the event range across the top of the slope (4105944: 0.03 s after the first position update inside it, at (-67.4, 33.4, -234.5)); the boss (14533; wall 2002872, line 2002735, PlaceName 690) and 14 helpers 2.1-2.4 s later; the Thaumaturge's 14360 32.4 s after |
| The clear | log 2014, scene 3 (185, 1, 18, 0, 0), coffer 112, the exit (2000139) shown; with Duty Support the three event NPCs 3.1 s later, in the Duty Finder three goblins 25.4 s later |
| The exit | a prompt; on yes the director's log 1534 `[8]` (the InstanceContent, as Haukke Manor's `[6]`), then the player out where they had queued (both runs: by quest 832's NPC in Eastern La Noscea) |

Kawari: `content/s1d2_re.lua`. The ContentNpc event is new (`ContentNpcEventHandler`, the director's `onContentNpc`),
as are `leave_boss_room` (the dragon; the second boss's add if the boss falls before it comes) and `map_effect`'s
state; `onCommence` now reaches a dungeon's script too, and a duty's exit sends retail's log 1534 for every
InstanceContent (`handle_director_messages`). The ambushes and the last boss's cutscene go by the layout's event
ranges: any player (`info.player1`...) inside one's box (its middle, turn and half-sizes, copied from planmap.lgb into
`EVENT_RANGES`), and for the cutscene also `onGimmickRect` if the server knows 4105944 as a GimmickRect, as Sastasha's
last boss's is.

| Layout object (planmap.lgb) | Where | Used for |
|---|---|---|
| EventRange 4000437 | (43.42, 7.76, -77.21), turned -1.366, half-sizes 3 × 3 × 4 | the first ambush |
| EventRange 4000429 | (61.4, 7.25, -84.34), turned -1.406, 4 × 3 × 12 | the second ambush |
| EventRange 4105944 | (-66.72, 35.36, -234.73), turned 0.251, 1.5 × 3 × 9 | the last boss's cutscene |
| EventRange 9059667 (LVD_Boss3_Gimmick_Remake) | (-26.44, 35.0, -235.98), a cylinder of 20 yalms (10 high) | the last boss's floor (`spawn_boss`'s arena): the party bots and party NPCs don't dodge off it. Retail's party went no further than 19.4 yalms from its middle in either run; Kawari's ran 27 out (Bob's bundle 20261104-204423) |
| PopRanges 11438223-11438226 (LVD_zone_01) | (82.7, 0.8, 27.5), (15.8, 7.4, -65.0), (-115.1, -2.4, 34.1), (-61.1, 34.0, -234.4): outside the four boss rooms | the shortcut's destination, the next as each boss is pulled (`update_shortcut`) |
| SharedGroups 9068345, 9070173-9070182 (LVD_Boss3_Gimmick_Remake, `sgbg_s1d2_a0_gmc01.sgb`) | the seven puddle spots | their map effects (`map_effect_at`) |

Not used: EventRanges 4000419, 4000434, 4186387, 4186389 and 3998421 (the entrance: InstanceContent 8 names it), the LVD_Faith ranges (9394119-9394121, Duty Support's),
and the gimmick layer's pop ranges by the boss walls (4086919-4086922, 7373172-7373182); `bg.lgb` has the rooms' own
geometry, nothing the director needs.

**The last boss's puddles** (`onActorCast`): seven spots, a map effect each (0-6), two helpers on each. With each 28656
(3.0 s, on itself: 16.2-16.3 s into the fight, then two 5.1 s apart every 48.2 s from 57.7-58.6) four or three spots'
helpers cast 28657 (5.0 s) at once; 5.5-5.6 s later those spots' map effects go to state 1 (timeline 2) and their other
helpers cast 28658 (9.0 s, 10 yalms); 8.9 s after that the map effects go to state 4 (timeline 8). The first 28656
marks the middle (0) and set A; each pair after it the middle and one set first (A in three of five pairs over the two
runs), then the other. Each spot is one of the layout's SharedGroups (within 0.04 yalms of its helpers), and the
director names its map effect by it (`map_effect_at`): the server finds its index in the duty's
ContentDirectorManagedSG (the list retail's DirectorSetupMapEffects follows), so no guess is left; the indices seen on
retail (the middle 0, A's among 2, 4, 6, B's among 1, 3, 5) are what's used if a layout ID isn't in the list. Its 28659 (5.0 s, lock-on 218, status 2104 for 15 s; 362 on the tank)
5.1 s in and every 41.7-48.4 s; 28660 (a cone) 36.6 s in, every 47.5-48.4 s.

**Coffers** (Treasure rows; one item each opening): 109 31476 or 31473, 105 4462, 106 3608, 110 3610 or 3609, 111 3382
or 3380, 112 31472.

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
knockback, the Knockback sheet's row 1 on every one hit (58 in 15 uses), that did 1; sent since 2026-11-11) at 15.0 s and every 41.6 s, Incinerate 7.9, 20.5 and 33.6 s after each. Below 80 % (79.9 and
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
Since 2026-11-07, as everyone is sent back, every enemy that was fighting or hurt goes back where it began, whole,
out of battle and with no one hated (`npc_behavior::reset_after_wipe`: an ActorSetPos, SetBattle 0 and no target from
each; its cast, and the director's `busy_bnpc`/`root_bnpc` hold on it, gone), and every sealed or sealing boss room
opens again (`reopen_rooms_after_wipe`: its wall hidden, log 2014, the pending seal dropped; its line stays while the
boss lives), and the party's NPCs come back beside the ones they follow at once, out of the fight (those still
standing had fought on, and the reset boss took one up again a second later: Bob's bundle 20261108-145830), their
actions and the reset enemies' under way called off (since 2026-11-10: one landing just after the reset put a bot back
on an enemy's list, and two 1672s came for the party's NPCs at the entrance at once, bundle 20261110-134029), as
retail's Stone Vigil did after each of its three wipes (log 2014 4.3 s after the fall, the bosses
whole where they began). Before, only a script's own `onPartyWipe` reset anything, and the Stone Vigil's first boss
stayed in battle and chased the party to the entrance (Bob's bundle 20261107-212704).
Since 2026-11-10 a Player NPC (a Duty Finder bot, shown as a player) goes back as retail's party members did, not
as its Duty Support members: three wipes in parties of players (Bob's Chronofoil captures 1aac5be2, 84cdbfb1 and
dd09ac61) had no DeleteActor or SpawnPlayer for any member; 3.3-3.4 s after the director's PartyWipe each member's
statuses went (ActorControl 20s), then ActorControl 212 with 1, their status list, an ActorSetPos with the player's
own warp type and argument (25, 2), ActorControl 2 with 1 (mode normal), 60 and 1529, and 0.35 s later ActorControl
200 with 1 (ZoneIn); no PartyList was sent again. The player's own conditions came with it, three times, without
Casting or InCombat (BoundByDuty and 28 alone). Kawari took the bots away and spawned them again, the likeliest cause of
the client's party list breaking (Bob's bundle 20261110-141944: "on reset the party list broke"; not yet played since): `party_bot::return_beside`
moves them now (`back_to_the_start` picks by kind; Duty Support's members are still spawned again, as retail's
were), and the players' Casting condition is taken off as they're sent back.
Since 2026-11-09 the player's position is the start's from the moment they're sent back (`do_change_zone` keeps the
ActorSetPos's spot): the server kept the spot they had fallen on until their client moved, so the party's NPCs got up
there, in the arena, beside the boss brought back (the Navel: 10 yalms from the new Titan, who took them up at once,
his Tumult 2.5 s after the reset and Rock Buster on one of them, before DutyRecommence; Bob abandoned the duty; bundle
20261109-195426). Those NPCs were also got up a second time at the recommencement (taken away and spawned again):
only those that fell since are now.


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

## The Navel

From Bob's Chronofoil captures of 2026-11-06, during Lord of Crags (quest 857, which unlocks it): `650dbfd8` visit 0.1
(5.7 minutes with Duty Support, Bob a level 36 arcanist), `0fd8bce0` visit 0.3 and `48aaf6fe` visit 0.2 (the Duty
Finder with three other players), each won at the first attempt. Read with `questkit chronofoil --battle` and the
visits' packets (numbers only), with Cactbot's titan-nm timeline (see [Sources](sources.md)) for what the runs don't
show. ContentFinderCondition 57 (ShortCode `s1fa_re`), territory 1046 (`ffxiv/sea_s1/fld/s1fa`), InstanceContent
20002: Bob's `content-index.json` (ControlPanel's *Build the content index*, 2026-11-06). The drop-in appends
`bg/ffxiv/sea_s1/fld/s1fa/level/planmap.lgb`.

**Enemies** (drop-in `s1fa_re_planmap.json`, all `nonpop`: the director spawns them; level 34): Titan (BNpcBase 246,
layout 4039257, 25124 HP) at (0, 0, -16) facing south, toward the entrance at (0, 0, 24) (players come in facing
-3.142, `ENTRANCE_ROTATION`); his helpers (434, all layout 4145900, hidden; four, as retail); the granite gaol (1444,
layout 4249552, 571 HP, spawned hidden); the Heart of the Mountain (1507, 5139 HP; no layout ID on retail, which
spawned it: the fork's 99001507).

**Titan** (`TitanNavel_246.json`, `from_pull`; the three runs agree to a tenth or two), his auto-attack 872 every 3.0 s
throughout (131 to the players' tanks, 71 to the Duty Support Marauder):
* *From the pull* (his line 3800, battle talk under BNpcName 1801, no portrait, 5 s): Tumult (642, round him, 175 to
  players) at 2.1 s and Rock Buster (641, on the tank, 357) at 8.3 s, each every 10.25 s.
* *The first leap*, below 80 % and not while he casts (untargetable at 79.3, 75.5 and 79.9 %, 0.9-1.1 s after his
  health fell below; with a Landslide being cast, 2.1 s after its hit): ActorControl 54 (untargetable) and
  ActionTimeline 140; 2.1 s on his line 3804, ActorSetPos to the middle facing south and Geocrush (651, 4.2 s, round
  the middle: 971-1017 to players 4.7 yalms out, 347-480 at 19-22, a line through all 17 hits: `by_distance`); 9.0 s
  after the leap targetable again. Cactbot's 85 % isn't what the runs show.
* *Phase 2*, from his being targetable: Tumult 2.1 s on, Rock Buster 7.3, Landslide (650, 2.7 s, a line before him,
  634 and the Knockback sheet's row 37) 13.6, each every 18.6 s.
* *The second leap*, below 50 % (49.4, 46.6, 49.4 %), the same, and 9.1 s after it the Heart: his line 3801,
  ActorControl 31 `[0, 0, 0, 1]` and 30 `[0, 1, the Heart, 0]` on him, ActionTimeline 1439, and the Heart at his chest,
  2.3 yalms before him the way he faces, moved with him as he walks and turns (`attach_bnpc`): retail sent Titan's
  ActorMove with each of the Heart's, both with his facing (the Heart's model sits in his chest only as long as the
  client has him facing that way). Since 2026-11-09 Kawari does too; it sent the Heart's alone, so when Titan turned
  to a target with no ActorMove of his own (Rock Throw at the player), the clients saw the Heart jump to his other
  side, out of his body, toward the player (Bob's bundle 20261109-195426). Retail spawned it as **a part of Titan**:
  BattleNpcSubKind Part (1), Titan's actor ID in the spawn's `pet_id`, no layout ID, display flags 0x40003, MP 1080,
  his facing, its target the player (all three runs); the client then draws and targets it in his chest. Kawari
  spawned it as an ordinary enemy, whose target ring the client put where it stood, on the ground before him (Bob's
  bundle 20261109-201914); since 2026-11-09 it's spawned as retail's (`director:spawn_bnpc_part`). He stays untargetable
  (and took no damage) but fights on: Rock Throw 7.9 s on (once), Landslide 14.2, Tumult 23.3, Rock Buster 28.5, every
  22.6 s. The Heart fell 24.7, 50.0 and about 27 s after it came.
* *The Heart's fall*: 30 `[0, 2, the Heart, 1]` on him; 3.2 s on ActionTimeline 140, 5.3 s ActorSetPos to the middle
  and 141, 7.3 s his line 3802 and Earthen Fury (652, instant, on everyone: 630 to players), 9.7 s ActorControl 31
  `[0, 0, 16, 1]`, 12.2 s 31 `[0, 0, 16, 0]`, targetable, his line 3803 and his last phase. If it still stands 73.6 s
  after the second Geocrush began (Cactbot's "Enrage"), Kawari takes it away and Earthen Fury comes the same way; what
  retail's does then isn't recorded.
* *Phase 4*, from his being targetable again: Tumult 2.1 s on, Weight of the Land 8.2, Rock Buster 15.4, Landslide
  21.6, Rock Throw 32.9, every 41.0 s (Cactbot's "Tumult x2" was one hit each time here).
* At 0 his line 3805, the closing cutscene (scene 3: 245, then 1, 18, 0, 0) and the duty complete; 2 s later he dies.
* The director's variables (DirectorVars, sequence 1 throughout, all three runs): 32, 0 at the start; 32, 1 with the
  first Geocrush's line, 33, 0 4.9 s later; 33, 4 with the second's, 39, 0 4.9 s later.

**Rock Throw and the gaols** (six throws): instant, on someone of the party other than his target (the timeline's
`other`), hurting no one; 2.8 s on status 292 on them (ActorControl 20); 4.0 s on the gaol spawned hidden where they
stand, 4.8 s on brought into view (Appear 142, targetable); the party breaks it and the status goes as it does
(ActorControl 21 `[292, 0, 0xE0000000]`, `director:lose_effect`). Gaoled, the Duty Support members did nothing for its
11-22 s: 292 is among the statuses that stop an NPC (`modifiers::INCAPACITATED`, party bots too). **A gaoled player**
(retail, captures 650dbfd8 and 48aaf6fe, the player gaoled each time, 13.7 and 8.8 s): as the status comes, ActorControl
54 (SetTargetable 0) on them, Condition with SufferingStatusAffliction and SufferingStatusAffliction2 (its first
byte 0x02 → 0x1A), then ActorControl 20 (292); as the gaol breaks, 54 (1), the conditions cleared and 21 (292). The
two conditions are what stop the client moving and acting; since 2026-11-09 Kawari sets them (and the targetability)
on any player with a status in `INCAPACITATED` and clears them as it goes (`server/mod.rs`); before, a gaoled player
walked out of the gaol (Bob's bundle 20261109-195426). Retail sent the same Condition (0x1A) for status 2955 too.

**Weight of the Land** (eight casts): his 644 (2.7 s on himself) and, as it begins, two helpers' 973 (3.2 s, round
them: 485 to players) under two of the party other than the one he fights, never his target. Retail aimed them at the
ground (TargetArea); Kawari puts a helper there and has it cast on itself (as Ifrit's).

**The Duty Support members** (the third set, 44, 47, 46 under ActorControlSelf 276 10008, 10011, 10010, at 34 with
1607, 1103 and 1113 HP: `party_npc::DUTY_LEVELS`): the Marauder's 14361 at the pull, the Conjurer's 14366 3.0 s after
the Heart fell; the Thaumaturge's 13975 three times and the Marauder's 13955 once were their chatter. They left Titan
alone while he was untargetable (their targets "no one" through each leap) and broke the Heart and the gaols:
`director:set_targetable(…, false)` now keeps bots off an NPC until it's targetable again
(`Instance::untargetable_npcs`). For both Geocrushes they went to the arena's edge, the same spots each time, 3-4 s
into the 4.2 s cast: the Marauder (0, -19), the Conjurer (0, 19-21), the Thaumaturge (19-21, 0) (capture 650dbfd8).
Geocrush covers the arena and hurts less the farther out, so nobody leaves it; the director gives these spots as being
inside it (`director:party_spots(…, true)`, [Party Bots](party-bots.md#how-it-fights)), and the party's NPCs and bots
go there (since 2026-11-09: before, they found nowhere out of it and stayed by the player, Bob's bundle
20261109-201914).

**What Titan deals, checked** (Bob's bundle 20261109-201914, two runs as a level 36 conjurer with 1035 HP, the party's
healer): Tumult 131-179, Geocrush 307-532 (the second time nearer him), Earthen Fury 606-655, Rock Buster 344 (once, on
him), as retail's. Retail's level 36 arcanist (capture 650dbfd8) had 963 HP, 999 with the party bonus (not given by
Kawari: [Known Issues](known-issues.md#the-navel-2026-11-06)); Earthen Fury takes about two thirds of that, so the party
needs to be near full health when the Heart falls.

**Leaving** after the clear during Lord of Crags puts the player by the beastman aetheryte in Upper La Noscea (the
quest's `set_duty_exit`, [Quests](quests.md)).


## The Howling Eye

From Bob's Chronofoil captures of 2026-11-08, during Lady of the Vortex (quest 519, which unlocks it): `6a5244ec`
visit 0.6 (Duty Support, Bob a level 44 conjurer), `09ecf8e7` visit 0.2 and `6c495fc4` visit 0.4 (the Duty Finder with
three other players), each won at the first attempt (149, 182 and 281 s from the pull). Read with `questkit chronofoil
--battle` and the visits' packets (numbers only); Cactbot has no timeline for it. ContentFinderCondition 58 (ShortCode
`r1fa_re`), territory 1047 (`ffxiv/roc_r1/fld/r1fa`), InstanceContent 20003: Bob's `content-index.json`. The drop-in
appends `bg/ffxiv/roc_r1/fld/r1fa/level/planmap.lgb`.

**Who's there** (drop-in `r1fa_re_planmap.json`, from `questkit chronofoil --dropin 1047`, all `nonpop`: the director
spawns them; level 44): Garuda (BNpcBase 239, layout 4246177, 44176 HP) at (1.14, -1.95, -18.0) facing south, toward
the entrance (players come in facing -3.14, `ENTRANCE_ROTATION`); her helper (1570, 4246419, hidden) in the middle; the
four monoliths (237, layouts 4246138-4246141, 7800 HP, battalion 0, display flags 0x40208: unseen and untargetable) at
about 8 yalms from the middle; the razor plumes (238, all layout 4246290) and satin plumes (688, 4246291), 200 HP.

**Garuda** (`TrialGaruda_239.json`, `from_pull`): auto-attack 870 (281 to players, 166 to Duty Support's members);
656 at 2.3 s and every 17.2, 657 at 9.1 and every 15.2, 658 at 16.6, 659 (2.2 s round her, a 4 s stun) at 31.8, all
three runs to a tenth. Every big move begins with her **vanishing**: untargetable, ActionTimeline 1415 (ActorControl
407), and 3.6 s later back (targetable, 1416) where it's cast from, the cast 0.7 s after, her timeline starting again
from then (anchor 1416: 656 3.9-7.1 s on, 657 10.8-12.4, 658 18.8-20.3):
* *Below 80 %* (1.7-3.4 s after her health fell below, not while she casts): **Mistral Shriek** (667, 3.7 s; 4.5
  once) from the north (-0.02, -1.85, -21.71), on everyone but those behind a monolith, and the monoliths themselves
  (1216 to players, 719 to a monolith). Duty Support's members gather at (-8, -4), behind the monolith at (-6.37,
  -6.36) and 0.5 yalms off the line through it, and were never hit; the Thaumaturge says 14382 as it begins. 13.0 s
  after it began her 1412 and six razor plumes where she stands, two at a time a second apart. While she's above 50 %,
  again 64.8 s after she vanished (the Duty Support run; the others were below 50 % by then).
* *Below 50 %*: her line 3701 at once (0.3-1.1 s after), 1.0 s on she vanishes; back, 1412 0.7 s on and eight razor
  plumes at the eight spots round the arena (about 20 yalms out), two every 2 s from 1.3 s on, in a different order
  each run.
* *Below 40 %* (0.6-0.7 s after, 3.0 once): back in the middle, her line 3702 and **Mistral Song** (662, 3.7 s, everyone:
  1061 to players, all twelve hits); her line 3703 11.0 s after it began; 16.1 s after it her 1411 and **Feather Rain**
  (664, her helper's, 2.7 s in the middle every 5.2 s to the end, which hit no one in 32 casts); 33.0 s after it
  **Aerial Blast** (660, 3.7 s) from the middle; 11.4 s after that 1412 and eight **satin plumes** at the eight spots, a
  second apart, each going for the player; 51.1 s after Aerial Blast began **661** (3.7 s) from the north. Only the
  Duty Support run got past Aerial Blast; what comes after 661 isn't recorded.
* At 0 her last line (3705), the closing cutscene (scene 3: 253, then 1, 18, 0, 0) and the duty complete.

**The whole fight, checked** (2026-11-11, on Bob's request to flesh it out): all three runs are duty-sim fixtures
with every battle talk, ActionTimeline and plume of the fight (`58-09ecf8e7`, `58-6c495fc4`, and the Duty Support run
`58-6a5244ec`, 35, 39 and 49 events), and the script does each within 3 s. What that brought out and what changed:

* **Her thresholds** are 80 % and 40 % as her health in whole percent reaches them (retail vanished at 80.1 % and at
  40.2-40.4 %, which `< 80` / `< 40` missed: the song came 18-20 s late), 50 % below it (her line after 49.0-49.1 %,
  never at 50.5-50.8 %).
* **At 40 % she can't be hurt** from her vanishing until her line 3703 (every hit on her shows Invincible, effect 7, in
  both runs that got there; her health stood at 40.2-40.4 % for 17-20 s.)
* **The monoliths**: below 65 % each shows its cracks (ActionTimeline 1713, once: four times in two runs, between 69.5
  % and 60.6 %); 5.5 s after Mistral Song began all four crumble (1715, all three runs to a tenth) and are gone 2.2 s
  later. After a wipe they stand again whole.
* **Her own timeline** after each return depends on what she did (all three runs, TrialGaruda_239): after a cast
  (Mistral Shriek, Mistral Song, 661; anchor 1416, from the cast's end) 656 at 7.7 s, 657 12.6, 658 19.4, 659 begun
  35.2; after coming back at 50 % with no cast (anchor 1412) 656 3.9, 657 10.8; after Aerial Blast (anchor 660) 657 at
  2.4, 659 begun 11.6, 656 17.7, 658 28.0. One set of points for all of them had 656 come 2-4 s early after a cast.
* **Eye of the Storm** (664, her helper's every 5.2 s from 16.1 s after Mistral Song): a ring 12-25 yalms round the
  middle (Boss Mod's T03GarudaN), the arena's edge, which its Action row doesn't draw (`action_areas.json`'s `rings`).
  Nobody stood further than 10.9 yalms from the middle in any of its 32 casts, and none was hit; whoever stands in it
  now takes a magic hit (1061, as Mistral Song: a guess, `TrialGarudaHelper_1570.json`). It was only an animation.
* **The monoliths are in her way, not her enemies**: retail's Garuda chose a player or a Duty Support member as her
  target every time; the monoliths took only her area attacks (656, 657, 658, 667) and the plumes' blasts (665), and
  the razor plumes' own targets. Kawari's went for one as soon as she was out (Bob's bundle 20261111-125206: "Garudo
  attacks the pillar"): nothing notices an NPC that can't be targeted now ([Combat](combat.md), aggro).
* **After 661** (one run, at 3 %) nothing more is recorded: here Aerial Blast again 33.0 s after 661 began, then the
  satin plumes and 661 as before, over and over (a guess from the spacing of Mistral Song and Aerial Blast.)

**The plumes** (`TrialRazorPlume_238.json`, `TrialSatinPlume_688.json`): no auto-attack; a razor plume goes for the
nearest monolith from where it came, a satin plume for the player; one left standing blows (665, round itself: 705 to a
monolith, about 200 to the party) 24.7 s after it came and falls 2.3 s later (ActorControl Kill; the director's
`kill_bnpc`). 3 of 42 razor plumes and 1 of 8 satin plumes did.

**Sheltering**: an area attack can spare whoever a living NPC of a given BNpcBase stands in front of
(`action_areas.json`'s `sheltered_by`, read by `gamedata::action_sheltered_by`; `server/action.rs`'s `is_sheltered`,
with a test): Mistral Shriek and the monoliths, 2.0 yalms either side of the line from her to them (a guess at their
width). The party's NPCs go behind one as she casts it (`party_spots`).

**The Duty Support members** (the third set, 44, 47, 46 under ActorControlSelf 276): the Conjurer's 14381 at the pull,
the Thaumaturge's 14382 as the first Shriek begins; the Marauder's 13955 and the Thaumaturge's 13978 and 13985 were
their chatter.

**Leaving** after the clear during Lady of the Vortex puts the player at Ul'dah's airship landing (the quest's
`set_duty_exit`, [Quests](quests.md)).

## The Stone Vigil

From Bob's Chronofoil captures of 2026-11-07: `1cefedf4` visit 0.1 (39 minutes with Duty Support, during In Pursuit
of the Past, quest 952, which unlocks it at its sequence 2; three wipes: at the first boss once, at the last twice)
`341808b0` visit 0.2 (16 minutes, Bob's tank through the Duty Finder with three other players) and `f842581d` visit
0.1 (17 minutes, his conjurer through the Duty Finder; the client crashed as it left by the exit), read with
`questkit chronofoil --battle` and the visits' packets (numbers only), and the public guides (Sources below), which
agree on the bosses' moves. ContentFinderCondition 11 (ShortCode `r1d1_re`), territory 1042
(`ffxiv/roc_r1/dun/r1d1`), level 41 (synced 43); the drop-in's LGB is `bg/ffxiv/roc_r1/dun/r1d1/level/planmap.lgb`,
which Bob dumped with `bg.lgb` (read with physis' `Lgb`, numbers only): the event ranges, pop ranges and map ranges
below are its. Retail's DirectorVars were the sequence 1 alone throughout; the duty's objectives went as the bosses
fell (ContentDirector 0x80000016 `[0, 1]` then 0x80000015 `[1, 0]`; `[1, 1]`, `[2, 0]`; `[2, 1]` at the clear: the
script's `objective`), BGM 37 as the first and second bosses were pulled and 0 as they fell (none for the last).
Kawari: `content/r1d1_re.lua`, drop-in `r1d1_re_planmap.json` (`questkit chronofoil --dropin 1042`: 48 battle NPCs),
timelines `StoneVigil*`, action scripts `StoneVigil*`.

**Seven more runs** (Bob's capture 216f28eb of 2026-11-17, visits 0.1, 0.4, 0.6, 0.8, 0.10, 0.14 and 0.16: Duty
Support, a Paladin synced to 43 with the Conjurer, the Thaumaturge and the Lancer, 19 to 22 minutes each, no wipes;
`duties/11-216f28eb-*.json`). What they added:

* **Isgebind's first round**: Rime Wreath 8.1-8.2 s after the pull, Frost Breath 9.1-9.2 s after it, the Sheet of
  Ice 17.3 s after it (16.4 in later rounds: the 16.4-17.5 below), and two looks at its health, 21.9 and 29.8 s
  after the Rime Wreath (30.0 and 38.0 s after the pull): below 85 % at the first it flew then (four pulls: 84.3 to
  84.8 %), else at the second (two: 85.5 and 85.0 % at the first), not as soon as it fell below. One pull was
  below 85 % before the Sheet of Ice and flew at once (25.4 s), as in the earlier runs. Flights are 44.1-44.3 s
  apart while they follow each other (14.0 s to targetable, the next round 4.1 s on, its decision 26.0 s after that),
  61.4-61.8 s after one with the Sheets of Ice.
* **Below 50 %**: at a Frost Breath's or Sheet of Ice's time it began a new round 4.0-4.1 s later instead (below
  before the Frost Breath: the next Rime Wreath 13.1-13.2 s after the last, two pulls; below 12.3 to 13.4 s into the
  round: 20.3-20.4 s, four), and it looks once between the two as well: below at 11.3 s, the new round came at 16.3 s,
  when the Sheet would have (one pull; see the five runs below).
* **The whirlwinds' helpers** (14544, 9061123) are there from the start: the four were sent with the second boss as
  the party neared it, 118 to 181 s before its pull, in all seven. The script brought them out at the first Typhoon;
  now in `onSetup` (`bringWhirlwindHelpers`), until the boss falls.
* **Lines without a tank of theirs**: [Party NPCs](party-npcs.md). `party_npc_yell`'s other role and line give the
  Thaumaturge's 14372 for the tank's 14371 and the Lancer's 14377 for its 14376; the start's is one of 14367, 14368,
  14369 and 14370, from whoever is there.
* **Five more the same day** (his capture d1f99707, visits 0.3, 0.5, 0.7, 0.9 and 0.11: a Summoner synced to 43
  with the Marauder, the Conjurer and the Thaumaturge, 20 to 21 minutes each, no wipes; `duties/11-d1f99707-*.json`)
  put three of these right:
  * **The first flight comes at one of Isgebind's moves, not as it falls below 85 %.** All five were below 85 %
    14.0 to 16.1 s into the first round and flew at 17.3 s, when its Sheet of Ice was due (25.4 s after the pull;
    so did the one of the seven above that flew "at once"). The script took off as it fell below, 1 to 3 s early;
    that path is gone, and the check at each move is what's left.
  * **Its look below 50 %** is 11.6 to 12.3 s into a round (below at 11.4 and 11.6 s in two of the five: the new
    round at 16.3 s). `LOOK_AFTER` is 11.6.
  * **The start's line comes as a player sets off**, 0.7 to 1.1 s after their first step in all twelve runs (Bob
    stood 8 s in one; he watched the opening cutscene for 29 s in the five), not 4.3 s after the duty began: the
    script says it when a player is 5 yalms from the entrance (`LINE_START_FROM`). The Marauder has one too, 14367
    (three of the five; the Thaumaturge's 14368 the other two).
  * **The dragons' and ice walls' ranges** are 4 yalms deep, and in one run the player was 0.36 yalms short of the
    first dragon's at one update and 0.24 past it at the next: `anyoneIn` counts halfway between a player's last
    two positions too.
* With those the script is within 1.6 s of all twelve (48 NPCs and 78-82 events a run: the perched dragons' and
  Isgebind's ActionTimelines, the ice sprites, the music, the objectives).

**The entrance.** Its entrance circle is EObj 2001869 (layout 4239683, layer LVD_gimmick), bound to the SharedGroup
4239670 at (0, 4.8, 306) where retail put the players (all three runs; hidden at DutyCommence, shown again after each
wipe), not one of the circles Kawari knew (2000182, 2001710, 2007457): with none found, every entry and every return
after a wipe went to a random pop range of the duty (Bob's bundle 20261107-205303: "teleporting in completely wrong
spots"), and the circle never hid. `ENTRANCE_CIRCLE_IDS` (core, `common/game.rs`) now has it too; players face
-3.1416 (`entrance_facing.json`).

**Enemies.** What the director brings is `nonpop`: the last boss (1455, 4001014), the second boss's whirlwinds
(14536, 9061122) and their helpers (14544, 9061123), the nine ice sprites (1465) and the last boss's eight helpers
(BNpcBase 9020, no layout ID on retail: the fork's 99009020). The four perched dragons (1460, not hostile) stand from
the start; the first two bosses (1461 at 4000969, 14535 at 9061121) are the layout's. New action scripts: 507, 518,
901-903, 1021-1027, 28730-28732, 29053; statuses from the captures' EffectResults: 902 267 for 15 s, 1026 268 for
15 s, 507 18 (50) for 30 s, 28731 2656 for 4 s; 1027 knocks back (row 1). Swinge (903) has no omen: 60 degrees wide
(`resources/data/action_areas.json`: it caught members up to 28 degrees off its facing and none at 40 or more).

| What | Retail |
|---|---|
| The perched dragons (1460; layer LVD_gmmick_wyvern) | each once, as a player first came into the range ahead of it (4240064, 4240075, 4240078, 4240080): ActionTimeline 7749, its 1021 (an instant line ahead) 1.96 s later, ActionTimeline 7739 2.09 s after that |
| The ice walls (EObjs 2001881-2001883; LVD_gmmick_ice) | as a player first came into the range in front of one (4240100, 4240136, 4240137): ActorControl 106 with 0 from it, and its three ice sprites 0.2-0.4 s later at it |
| The first boss (1461; wall 2001873, line 2001872, PlaceName 877) | Swinge 14.4 s after the pull (14.2-14.6, three pulls), then about every 31 s (26.6-36.6); before each it runs (ActorMoves every 0.31 s, 2.8 yalms apart, speed byte 90: `sweep_bnpc` at 9 yalms a second; a single ActorMove to the spot, `move_bnpc`, snapped it there on the client, Bob's bundle 20261107-212704) to one of three spots at the room's edge ((0, 130), (15, 116), (-15, 116); never the one it's at) and casts as it arrives, facing into the room (all 13 first Swinges 0.02-0.72 off the way to the middle, (0, 116): at its target, kept within that), and keeps facing that way through the cast (`sweep.rs` sends nothing more once the run has reached its end: one more ActorMove facing the way it ran turned it back to the wall on the client, so the cone the client and Bob's plugins drew pointed away from the one that hit, Bob's bundle 20261110-134659); below half its health it Swinges again from there 6.3-6.4 s later at its target (every double at 45-49 % or below, every single at 68 % or above). Lion's Breath (902) and Rake (901) its timeline's. Swinge spared those beside or behind it, inside its hitbox, as every cone does now ([Combat](combat.md)); Lion's Breath (0.7 s) hit its target 24 times of 25, and nobody dodges it ([Party Bots](party-bots.md)) |
| Its death | log 2014, the way on (2001878) state 7, coffers 179, 180 and 182, the tank's 14371 (the Thaumaturge's 14372 with a player tank) |
| The second boss (14535; wall 2002872, line 2002735, PlaceName 878) | its round of 44.9 s: Typhoon, Spiked Tail (28732) 30.2 s later, Sonic Storm (29053) 37.4 s later; the first round shorter (Spiked Tail 5.1-5.2 s after the pull, Sonic Storm 12.4, Typhoon 20.5-20.7, the next 37.7 s after it), both runs |
| Its whirlwinds | 1.5 s after each Typhoon ends, at x 57 in lanes z -72, -77.5, -82.5, -88, a helper riding with each (ActorSetPos to it, then its 28731, a 3-yalm circle, every 0.58 s from the first pair's setting off until its own arrives). Two set off 3.9 s after they came, the others 7.1 s after them, at 3.0 yalms a second (ActorMove every 0.31 s, speed byte 30) to x 30.4; gone 23.5 and 30.5 s after they came. Each hit someone once (damage, status 2656 for 4 s, which throws them into the air, and an effect of kind 62 with 173, 64) and left them alone while 2656 lasted (all eleven hits in the three runs: `action_areas.json`'s `spares_while`). The first Typhoon's were three: -72 and -82.5 then -88 (the first two runs), or -77.5 and -88 then -82.5 (the conjurer's); after it four, in pairs: -72 with -82.5 and -77.5 with -88, either first, or -72 with -77.5 and -82.5 with -88, either first (all four orders seen over the three runs) |
| Its death | log 2014, the way on (2001879) state 7, coffers 181, 183 and 184, the Thaumaturge's 14375 |
| The last door (2001880; GimmickAccessor 0x000F00DB) | EventAction 24 for 1.9 s, state 7; the room's cutscene (249, params 1, 38, 1, 0) as a player crossed the range past it (4239894), the boss and its helpers 2.2 s later |
| The last boss (1455; wall 2001877, line 2001876, PlaceName 879) | its round on the ground: Rime Wreath (1025), Frost Breath (1022) 9.2 s later, Sheet of Ice (1023, on one of the party) 16.4-17.5 s after the Rime Wreath, its flight (or the next round) 26.0 s after it (30.1 in the round after the pull; the first Rime Wreath 8.1-8.2 s after the pull, the next 4.1 s after it lands or doesn't fly) |
| Its flights | the first as soon as it was below 85 % with its round's Frost Breath or Sheet of Ice still to come, once it wasn't casting (the tank's and conjurer's runs: 0 and 1.8 s after it fell below), else at the round's end (2.8-7.1 s after, three pulls); one more between 50 and 70 % at a round's end (59.7-66.0 % three times; at 48-55 % it didn't, three times); then at every round's end below 40 % with the Sheets of Ice. Falling below 50 % with a move of its round still to come, it began a new round at that move's time instead (the conjurer's run: a Rime Wreath 13.2 s after the last, no Frost Breath or Sheet of Ice between); in the other pulls it fell below 50 % after its round's last move. SetTargetable off and ActionTimeline 1385; over a dive's spot 4.1 s later (ActorSetPos, ActionTimeline 142 0.1 s later, Cauterize, a 3.7 s line across the room, 0.7 s after), the next dive 7.8 s after; home (0, -248) 7.8 s after the last, Touchdown (1027) as it lands, targetable 2.2 s later. With the Sheets of Ice its helpers cast 1024 (2.7 s) at each of the party 5.8 s in and again where they stood 3.0 s later, and two dives from 13.7 s in. The dives: (23.7, -262.5) and (23.7, -233.5) facing -1.57, (0 or ±14.5, -271.8) facing 0, at random, the two of a flight different |
| The clear | log 2014, logs 9393 `[75400]` and 2163 `[8400, 5100]` ([Clear rewards](#clear-rewards)), cutscene 300 (1, 18, 0, 0), coffer 185, the exit (2001871) shown; leaving it during quest 952 sent the player to New Gridania (the quest's `set_duty_exit`, [Quests](quests.md)) |
| Wipes | everyone back at the start; the bosses whole again; after the last door the last boss back in its room at once (1.9 s, both times). With Duty Support it's a wipe as the player falls, the members still standing (1cefedf4: 489.5 s in) |

| Layout object (planmap.lgb) | Where | Used for |
|---|---|---|
| EventRanges 4240064, 4240075, 4240078, 4240080 (LVD_gmmick_wyvern) | ahead of each perched dragon, half-sizes 8 (6) × 6 × 2, the last three turned 1.5705 | the dragons' breaths |
| EventRanges 4240100, 4240136, 4240137 (LVD_gmmick_ice) | in front of each ice wall | the ice sprites |
| EventRange 4239894 (LVD_gimmick) | (0, 7.42, -226.05), half-sizes 10 × 8 × 2 | the last room's cutscene |
| EventRanges 9394127, 9394128 (LVD_Faith) | the second boss's way in; the last room's floor before the boss | the Duty Support members' lines 14374 and 14376 |
| PopRanges 11438278-11438280 (LVD_Zone_01) | outside each boss's room | the shortcut, the next as each boss is pulled |
| MapRanges 4322871, 4322974, 4322979 (LVD_Maprange_01) | round each boss's room | the floors the party keeps to while dodging (`boss_room`/`spawn_boss`'s `arena`) |

Not used: the other LVD_Faith range (9394126, by the entrance), PopRange 6543370 (in the last room), the LVD_Faith
layout's four event NPCs (none came after either clear), and ContentDirector event 0x40000007 (the duty's time flags, sent since 2026-11-11: above).

**Since 2026-11-10** (Bob's bundle 20261110-182408, on `b0a1044b`): the whirlwinds throw whoever they hit
(`effects:toss(64, 173)` in `StoneVigilHelper_28731.lua`: kind 62 in the hit and ActorControl 220 from the one hit,
[Combat](combat.md): tossed into the air; before, only the status was given, and nobody left the ground), and the
last boss's placed attacks show where they're aimed, not in the middle of its room ([Combat](combat.md): where a
cast is aimed). His "can't execute command whilst casting" after a reset there was the death mid-cast fixed in
`315a808`, which his build didn't have.

**The Duty Support members' spots** (all from the first run, every time the same): for Swinge, Cauterize, the
Sheets of Ice and the whirlwinds, as [Party NPCs](party-npcs.md#getting-out-of-area-attacks) gives them; the script
hands them to the dodging (`director:party_spots` in `onSetup`, `director:party_positions` as each flight and
Typhoon goes), and the whirlwinds go by `director:sweep_bnpc` ([Party Bots](party-bots.md#how-it-fights)).

**Coffers** (one item each opening, the first run's; the others' where the coffer is known): 179 7033, 180 31530, 182
10500, 181 31533, 183 31518, 184 31525, 185 10438 and 7033, 7033 and 2176, or 7033, 31528 and 1927. The tank's run's
first boss gave 10438 and its second 31518 and 31527, the conjurer's 1927, and 31519 and 31520, from coffers not told
apart. The conjurer's run's exit can't be read (the capture ends in the loading screen to New Gridania).

## Castrum Meridianum

ContentFinderCondition 15 (ShortCode `w1d4_re`), territory 1043, director 0x8003000F, during Rock the Castrum (quest
3873). Built on 2026-11-17 (Bob: "start on the bosses now … between 3 runs") from Connor's Chronofoil capture
af05916c (visit 0.8, a healer; his party fell twice at the last boss) and Bob's 9ba7d981 (visit 0.12, a damage
dealer) and 50e72631 (visit 0.3, a healer), all with Duty Support's fourth set. **Untested in game, and built without
the duty's layout**: its event ranges aren't known, so what the party sets off by walking somewhere is set off by
where they were when it happened in the runs (the script's `WHERE_*` lines; Bob's two runs agree to a yalm or two).

**Who's there** (drop-in `territory_1043.json`, `questkit chronofoil --dropin 1043` over the three captures: 82 battle
NPCs). The enemies on the way stand from the start, as the three runs met them; `nonpop` (the director's) are the
first boss's three waves, the five that come out of hiding, the last boss and the helpers. Their timelines
(`CastrumMeridianumEnemy_<base>.json`, sixteen kinds) are what each did in the runs (`questkit sources --duty 15`).
Five non-hostile ones (BNpcBases 6358, 14580, 14618, 14671, 14672) stand as retail's did and do nothing; 14580
(BNpcName 1486) is who the duty's talks are under.

**The way** (all three runs; seconds are 50e72631's, from the duty's start):

| What | When |
|---|---|
| Talk 33506 (with the director's 0x80000007 `[42]`), 33507 7.0 s on | As the party sets off: 8 yalms from the entrance (1.4 s; 31.1 s in the run Bob stood) |
| Map effects 2 and 3 (timeline 2, state 2) | West of x = 208 (6.9 s) |
| Map effect 0 (2, 1) | The six of layouts 9088382-9088387 dead (122.2 s) |
| Map effect 1 (2, 1), 2 and 3 (4, 4) | The five of 9088400-9088404 dead (246.2 s) |
| Objective 0 done | Into the first boss's room (252.9 s, 2.2 s before its pull) |
| The first boss: music 37 at its pull, its wall (EObj 2000587) shut 15 s on (EventLogMessage 2012 with PlaceName 434) | 255.1 s |
| It fallen: music 148, 0x80000007 `[42]`, objective 1, talk 33508, map effect 4, the wall and line (2000588) open, EObj 2000597 usable (SetTargetable, state 0); talk 33509 7.0 s on | 445.5 s |
| EObj 2000597 used (EventAction 23, 1.0 s): PrepareZoning and everyone at (-11.0, 56.4, 205.9) | 452.8 s |
| Talk 33510 (0x80000007 `[42]`) | South of z = 230 after it (467.0 s) |
| Map effects 8 and 9 (2, 2) | West of x = -36 there (473.2 s) |
| Map effect 5 (2, 1) | The three of 9088917-9088919 dead (589.4 s) |
| Map effect 6 (2, 1), 8 and 9 (4, 4) | The two of 9088929 and 9088930 dead (711.5 s) |
| Objective 2 done; music 37 at the second boss's pull 2.0 s on, its wall (2000589) shut 15 s on (PlaceName 433) | 719.9 s |
| It fallen: music 148, 0x80000007 `[42]`, objective 3, talk 33511, map effect 7, map effect 10 2.1 s on, talk 33521 7.0 s on | 856.5 s |
| Three come out of hiding by the first boss's room (9088949-9088951: spawned unseen, Appear with ActionTimeline 142 0.7 s on, targetable a second after that) | Someone north of z = -18 there (869.3 s) |
| Talk 33512; 3.0 s on map effect 11 and EObj 2007457's ActorControl 409 `[1, director]` | The three dead (950.7 s) |
| Talks 33513, 33514 | Past z = -77, then west of x = -42 (957.8, 968.1 s) |
| Two more come out (9088956, 9088957), talk 33522 as they appear | Three of the six of 9088952-9088955, 9162432, 9162434 dead (1024.1 s) |
| The cutscene: scene 3 with cutscene 275 `[0, 151, 0, 0]`; EObj 2012983 opened; map effects 12-14 a second on; the last boss and its 50 helpers there | 2.1 to 2.4 s after the last of **all eight** fell (1099.3 s). The two alone looked enough in three runs; in the fourth (6320ae56) one of the six outlived them by 4.9 s and the cutscene waited for it |
| Everyone at (-83.9, 72.0, -35.2) (PrepareZoning, ActorSetPos); objective 4 done 1.8 s on | The cutscene's end (1213.1 s) |
| The last boss (wall 2000593, line 2002735, PlaceName 428); it fallen: the way out (EObj 2000596), the duty complete, scene 3 with cutscene 277 `[1, 18, 0, 0]` | 1221.1 to 1431.4 s |

No coffers came in any of the three (Duty Support, the story's first clear). Retail's objectives are five (0 to 4);
nothing is marked at the clear.

**The first boss** (9060841, BNpcBase 14538, 66977 HP; `CastrumMeridianumBoss1_14538.json`): one clock from its pull,
talk 33523 (the three runs to 0.3 s): 28773 at 5.3 s, 28774 at 14.6 (talk 33524), 29268 at 23.0, 28775 at 30.3, 28774
at 40.8 (talk 33525), 28773 at 69.1, 28774 at 78.3 (talk 33526), then 28773, 29268, 28775 every 31.5 s from 106.6,
119.0 and 129.5. Four soldiers stand with it; each 28774 brings a wave a second after it lands: six (three of base
14539, three of 14540), two of base 14668, then six (two of each). 29268 is followed by three helpers' 29269 on
everyone (286 to a player) 5.1, 5.6 and 6.2 s after it begins; 28775 lands where someone stands. It fell at 192 to
197 s.

**The second boss** (9060849, BNpcBase 14541, 77022 HP; `CastrumMeridianumBoss2_14541.json`): its first 28778 comes
10.6 to 11.4 s after its pull and the rest follow it to 0.5 s: 28780 at 17.4 s, 28785 (on its target) at 27.5, then a
round of 53.8 s: 28778, 29146, 28783, 28780, 28785 (41.1, 53.3, 60.8, 69.4, 82.6 s). A helper's 28779 (7 s, a circle
of 20: 743) comes from one of three spots ((-5.1, 19.3), (-20.9, 42.7), (-24.7, 23.1): no order to them) 4.9 s into
each 28778; helpers' 28781 and 28782 (a circle of 10 and a ring from 12.5: 1288) from the middle with each 28780; a
helper's 28784 (a circle of 5) under each of the party 6.4 s into each 28783.

**The last boss** (9060634, BNpcBase 14542, 119560 HP; `CastrumMeridianumBoss3_14542.json`): one clock from its pull,
talk 33515, in five pulls (to 0.2 s): 28033 at 5.1 s, 28788 (it goes, and is in the middle 0.7 s later) at 17.3,
28786 at 19.0 with eight helpers' 28787 on a ring of the room and 29163 in the middle, 28798 (on its target) at
32.5, 28791 at 45.9, then from 91.3 s a round of 74.2 s (28788, 29357, 28789, 28798; 28788, 29357, 28795; 28788,
28786; 28033). After 28791: **four volleys** 2.5 s apart from 47.4 s, each a helper's 28793 at the room's edge and
six 28794s in three pairs 0.2 s apart marching across the room (four fixed lines; the first two came in either
order), the boss showing itself (ActionTimeline 7747) four times from 55.5 s and using 28792 as each lands; two
28797s in the middle at 61.9 s (its 28796 at 63.9) and with each 28795. With 28789 six 28790s (three sets of spots
seen) and 29164 in the middle. 9.6 s into each 29357, 29356 from two opposite spots of the ring (any pair), four the
next time, two the time after. **Below 20 %** (the talk came at 20.0, 15.6, 20.0 and 11.7 %: at once when it was
free, 3.1 and 3.3 s after its 28798 when it was casting) talk 33520 and its last part: 28788, 29354 at 1.9 s, 29352
at 8.0 and 25.3, three 29353s at 14.8 to 18.5. It fell 19 to 31 s into that.

**A fall with Duty Support** (Connor's two, and Bob's two as a Paladin on 2026-11-18, capture 6320ae56 with Kawari
Witness): no PartyWipe or DutyRecommence came from the director. The player's fall (ActorControl 14, SetMode 2,
ActorControlSelf 511 `[1]` and 512); 3.0 s later the screen faded (ActorControlSelf 263 `[1]`), and 4.5 s after the
fall the player was **at the duty's entrance** (248, 87, -94), whole again (ActorControlSelf 236, then 200, 511
`[63]`, 263 `[0]`, 253, 260 and three 276s; SetMode 1 3.4 s on), the members revived with them (ActorControls 266 at
+2.9 s, then SetMode 1, 60 and 1529 on each at +4.4 s). Both times he walked to **the shortcut by the entrance** (an
object whose event is GimmickAccessor 74: scene 1, flags 0x2001, 1.4 s) and it put him outside the last room at
(-67.9, 69.5, -38.4) facing -1.396, the members at his side; the boss and its 50 helpers were new 0.4 s after he came
(11.5 and 9.7 s after the entrance; Connor's 31 and 73 s were his walk too). Kawari's wipe (`server/wipe.rs`) sends
everyone to the entrance as in any duty; the script takes the boss away, brings it back at `onDutyRecommence`, and
from the last room's cutscene on points the shortcut at that spot: **`director:update_shortcut_spot(x, y, z,
rotation)`** (new: the shortcut's `update_shortcut` takes a pop range, and this duty's aren't known;
`DirectorData::shortcut_goes`, test `a_shortcut_goes_where_it_was_last_pointed`). **When it shows** (2026-11-19): the
shortcut is EObj 2000603 (layout 9088364, at (241.5, 87.4, -90.7) by the entrance), and retail spawned it hidden
(state 7) in all five runs, still hidden 414 s in, shown (state 0) by 1308.8 s and 1529.3 s, after the last room's
cutscene: what `update_shortcut_spot` does. Kawari only knew 2000700 as a shortcut and showed 2000603 from the start
(Bob's mark: "shortcut showing when it's only suppose to show once reaching the last boss"). A duty's shortcut is now
2000700 or any EObj whose event is the same GimmickAccessor as 2000700's (`zone::is_duty_shortcut`; 74, which
2000603's is too): spawned hidden, all of them shown by `update_shortcut`/`update_shortcut_spot` (`ShowShortcuts`,
`Zone::shortcut_eobjs`), and taken where the director points. Where it goes before the last room isn't known: it was
only used there.

**The fourth run** (Bob's 6320ae56, visit 0.1, 2026-11-18: a Paladin, so the enemies went for a player; 1963.9 s,
two falls at the last boss) fits the three otherwise: the first boss's casts to the round's 31.5 s through 196 s, the
talks, waves and objectives. New from it: the cutscene's eight (above); the fall and the shortcut; **BNpcBase 14574's
29178 is a heal** (747 on one of its own side at 4 % of its health, a 1.2 s cast begun with it at 4 to 9 %; it had
been read as an attack that hit no one; when and on whom is the fifth run's, below); the soldiers' status 290 (28777) has no timer; and what the attacks that
had hit no player do to one: 29170 114-157, 29169 136-188, 28785 939 and 1044, 28798 1202-1620 (2255 to the members
it turned on), a helper's 28787 802. Those are a Paladin's: he took 1/1.75 of what Bob's damage dealer and healer
had from the same enemies' auto-attacks and about 1/1.5 from the duty's spells, so the timelines' `players` amounts
for them are the Paladin's times that (236, 298, 1488, 2112, 1203), an estimate.

**The fifth run** (Bob's deea324c, visit 0.1, 2026-11-19: a White Mage of 2657 HP, "ran it very poorly … dragging
out a terrible play … so you could see how much damage a healer takes"; 2096 s, the bosses alive 245, 149 and 239 s,
one fall at the last) fits the script as it stood (`15-deea324c`: 163 events). What the long fights showed:

* **The first boss's round goes on, 31.3 s long**: 28773 seven times to 232 s, 29268 six and 28775 five, 31.1 to
  31.6 s apart (31.5 in the timeline until now, from two rounds). The second boss's 53.8 s round and the last's
  74.2 s round each came a second time, as written.
* **The last part past 25 s**: 29352 again 17.4 s after the first and the three 29353s 17.3 to 17.4 s after theirs,
  as guessed; the part ran 40 s here (19 to 31 s before).
* **When BNpcBase 14574 heals** (29178; three uses in the five runs: 736, 747 and 715, always by the one of layout
  9088912, whose twin 9088910 falls first). Not by an ally's health alone, as the fourth run had it read (below
  10 %): the cast began 43.3, 43.4 and 45.3 s after its own first cast, in the first of its spell's turns from 43 s
  on, though its twin had been at 6 to 8 % from 25 s; the two that lived 42 and 45 s never used it. On whom: allies
  at 9, 21 and 31 % of their health, the last with another at 23 %, so not the most hurt; each time the lowest
  layout ID of those below half (the others stood at 59 % and more). Not again in the 19 s one fought on with allies
  at 3 to 34 %. So the timeline's entry is `not_before` 43, `ally_hp_below` 50 (the first of them by layout ID) and
  `every` 43, the last a guess ([Combat](combat.md)).
* **What a healer took** is what the timelines' `players` amounts already were (a damage dealer's: 28779 717 for
  743, 28794 1210 for 1170, 28033 907 for 937, the soldiers' auto-attacks within 5 %), so only a tank takes less.
  New amounts, where an attack had hit no player or only the Paladin: 28781 1279, 28784 1298, 28787 1176 (the
  estimate from the Paladin's was 1203), 29173 552, 29174 524, 29181 653, 29182 589, 29183 644, 29184 569.
  28785, 28798, 29169 and 29170 still have only the Paladin's, times 1.5 or 1.75.
* **Stun**: the last boss (14542) refused it (Holy, 13 s in; Kawari Witness's `refuses:2`), so it's with the kinds
  that refuse outright; the other 172 Holys on enemies went as `status_resistance.json` says but three (refused
  2.4 s after a quarter, the known edge), and an enemy still refused 39.5 s after its last.
* **The fall** at the last boss was the fourth run's again: no director wipe, the screen's fade 3.0 s after he
  fell, the boss casting until then (fixture `wipes`, by hand).

**Not known, so guessed or left out** (also in `known-issues.md`): the event ranges; the rounds past what the runs
show (they go on); the last part past 40 s (its casts every 17.3 s, seen twice); when 14574's heal comes again;
where 28788 takes the boss the once it wasn't
the middle; what the wave soldiers' 28777 status (290) does; what 0x80000007 `[42]` and EObj 2007457's ActorControl
409 are; the damage of 28773 and 29146 to a player (29146 hit a member for 1098 in the fourth run); the
shortcut before the last room.

**Tests**: duty fixtures `15-af05916c`, `15-9ba7d981`, `15-50e72631`, `15-6320ae56` and `15-deea324c` (`questkit
duty-sim`: 224, 105, 105, 225 and 163 of retail's events: the talks, music, objectives, every NPC brought out, the last boss's
ActionTimelines, the clear; and a rehearsed wipe at each boss). Connor's and Bob's two falls each are in their
fixtures by hand (`wipes`: the fall and the boss's coming back), as the director sent nothing for them;
`a_shortcut_goes_where_it_was_last_pointed` (`director.rs`), `a_heal_waits_for_its_time_and_an_ally_hurt_enough` (`rotation.rs`), `some_enemies_refuse_stun_outright`
(`resistance.rs`).

## The Porta Decumana

ContentFinderCondition 830 (ShortCode `w1d5_battle`), territory 1048, director 0x80034E72: the trial at the end of The
Ultimate Weapon (quest 4522), two fights with a cutscene between them. Built on 2026-11-17 from two retail runs with
Duty Support, both won: Connor's Chronofoil capture af05916c (visit 0.12, a healer with the Marauder, the Thaumaturge
and the Lancer) and Bob's 9ba7d981 (visit 0.18, a damage dealer with the Marauder, the Conjurer and the
Thaumaturge); `questkit chronofoil --battle` and the visits' packets. **Untested in game.** A third run the same day
(Bob's 50e72631, visit 0.7, a healer again) was read against the script: it kept the same clock, and corrected three
things, marked "third run" below.

**One clock from the pull.** In both fights every cast, instant, talk, map effect and ActionTimeline of the two runs
came at the same second from the pull (to 0.3 s), up to three places where the boss's health decides. So the script
(`content/w1d5_battle.lua`) is that clock: tables of `{ seconds, what, … }` rows (Connor's run's numbers) run from
the pull, the bosses' own casts in their timelines (`PortaDecumanaBoss1_14587.json`, `PortaDecumanaBoss2_14592.json`,
`from_pull`), everything round them the director's.

**Who's there** (drop-in `territory_1048.json`, from `questkit chronofoil --dropin 1048`, all `nonpop`):

| Layout | Base | BNpcName | HP | What |
|---|---|---|---|---|
| 9088367 | 14587 | 2137 | 262421 | The first boss, at (-781, -600) facing east; the arena is 20 yalms round (-772, -600) |
| 9088369, 9088368, 9088371 | 14589, 14588, 14591 | 2140, 2141, 2139 | 2589 | Three that stand unseen in the middle (display flags 393224), each shown by an ActionTimeline for its casts |
| 9088370 | 14590 | 1804 | 2589 | Four, brought in for one long cast |
| 0 | 9020 | 2137 | 44 | Helpers: 25 in the first fight, ten in the second on retail; 13 here (twelve cast at once at the most) |
| 9088373 | 14592 | 2137 | 244634 | The second boss, at (-713, 480); the arena is 20 yalms round (-704, 480) |
| 9088374 | 14593 | 2142 | 2589 | Ten at a time: the rows |
| 9088375 | 14594 | 2138 | 2589 | Four at the second arena's corners |

**The first fight** (seconds from the pull, talk 33200; the boss's auto-attack 29004 every 3.0 s throughout):

| When | What |
|---|---|
| 8.1 | 28628 (no cast): the boss goes, and comes out 1.7 s later at (-784, -600) facing east (SnapNextMove). The same at 103.4, 158.5, 233.2, 284.3 and 5.1 s into the chain |
| 10.5 | Talk 33207, its 28977 (4.7 s), ActorControl 70 `[8201055]`; base 14589 shown (ActionTimeline 7747) and its 28998 from the middle at 11.3 (1674-1723 to a player), hidden at 21.3 (7737) |
| 25.7-38.3 | 14589's 28999 (4.7 s, 546-589) then 29000 (2.7 s) from a spot on the edge, 15 yalms out; the boss's 28981 (2.7 s, facing someone) at 37.8; again at 66.0-78.5 from another edge spot |
| 47.0, 51.1 | Ten helpers' 29001 (2.7 s) on ten spots, then on ten others; again at 87.4 and 91.5; the two sets in either order each time (third run: the other set first at 47.0) |
| 105.8 | Talk 33208, its 28987 (3.7 s), a helper's 28988 on each of the party |
| 111.3-135.1 | Four of base 14590 round the middle (Appear with ActionTimeline 142 0.6 s on); from 116.0 their 28989 and the boss's 29435 (32.7 s casts); 119.9 map effect 4 (timeline 2, state 1), weather 105 and the lines under BNpcName 2183; 132.0 a helper's 28511 on the party and 28992 on itself; 132.9 the map effect off (8, 4) and weather 2; **134.9 both casts stopped** (ActorControl 15 `[537, 1, 29435]`, `[540, 1, 28989, 1]`) and the four fall; 135.1 its 28994 (status 3010 off it) and 14589 gone for good (ActionTimeline 4575) |
| 160.6-226.5 | Talk 33201, its 28976, ActorControl 70 `[8201054]`; base 14588 shown (4561) and hidden (4573) round its 28996 from the middle (162.0: 1593-1715) and three 28997s from edge spots (180.2, 195.2, 219.4); the boss's 28979 at 173.9 and 213.1, each followed by a helper's 28980 from the middle every 5.1 s (five, and four before the boss goes) |
| 235.7-263.0 | Its 28984 (status 3012 on it), then 28510 (27.7 s) at 244.8: the map effect and weather at 249.7, the party's 28511 at 259.8, **the cast stopped at 262.7**, its 28993 (status 3009 off) and 14588 gone (4576) |
| 286.4 | Talk 33213, its 28978, ActorControl 70 `[8201056]`; base 14591 shown and its 29002 from the middle at 287.8 (1625-1735) |
| 301.7 on | **The rounds**: its 28982 (1.9 s) at 301.7, 306.0 and 322.3, with three helpers' 28983 15 yalms from the middle and one in it (three ways round seen) the first two times and eight 17 yalms out the third; 14591's 29003 (533-540, and everyone thrown to the edge: Knockback 67) 14.9 s into the round (shown 1.3 s before it and hidden 6.1 s after it began: once shown it hides even if the chain has begun meanwhile, as in the fourth run, 0.4 s after the chain's talk). The round again every 40.9 s |
| below 36 % | Once the rounds have begun, whatever the round is at (**third run**: it began a second round at 36.7 % and the talk came 9 s into it, between its second and third 28982; the first two runs, read alone, looked like a 40 % line asked only as a round ended or began). The talk came 1.6 s after 35.95 %, as 36.3 became 35.97 %, and 3.2 s after 35.94 %, at 342.6, 367.5 and 356.4 s. **The chain**, from talk 33214: 28628 at 5.1 s, talk 33215 and its 28990 (31.7 s) with twelve helpers' 28991 at 7.6, the map effect and weather at 17.5, the party's 28511 at 26.6, the casts stopped at 29.5, its 28995 (status 3011 off) at 29.6 and 14591 gone (4578), talk 33218 and ActorControl 70 `[8201058]` at 41.8 |
| below 22 % | (21.96, 21.7 and 21.9 % 0.1 to 2.4 s before it, 62.0, 71.1 and 50.0 s into the chain.) **The cutscene**: the director's scene 3 (flags 0x08502403) with cutscene 120 and `[0, 150, 0, 0]`; every NPC of the first fight is taken away as it begins |

**Between the fights**: as the cutscene ends (279 and 281 s in the runs) PrepareZoning to the same territory and an
ActorSetPos put the player at (-689.55, -185.73, 480.35) facing west, the members beside them (`warp_players`; the
members catch up as party NPCs do), and the second boss and its helpers come 3.9 and 1.9 s later. A wipe from then on
sends everyone back there (`director:set_start`, new; retail's wasn't seen).

**The second fight** (seconds from its pull, talk 33219, which came 39.4 and 10.4 s after the cutscene's end):

| When | What |
|---|---|
| 8.1 | Its 29023 (4.7 s) on its target: 920-1285 to the tank. Again at 200.0 and 303.2 |
| 16.5, 18.9, 21.2 | Its 29005, 29006, 29007 (no cast), each with a helper's 29008, 29009, 29010 (1.9 s) from where it stands: **29008 the way the boss faces, 29009 45 degrees to one side, 29010 45 to the other** (all nine sets of the three runs; the third run showed the pattern, and 29010 before 29009 once). Again from 170.4 and 273.6 |
| 27.7 | Its 29011 (3.7 s) with a helper's 29012 (4.7 s: 925-1001) on each of the party. Again at 133.5 and 236.6 |
| 38.8 | Talk 33220, ActorControl 70 `[8201057]`, its 29022 (4.7 s: 907-1046 to everyone). Again at 157.2, 207.1, 260.3 |
| 51.0-67 | 28628 to the middle; 29015 (3.7 s) throws everyone to the edge (Knockback 13, no damage); 0.8 s after it lands **four of base 14594** at the corners (14.1 yalms out each way), which set off for the middle 5 s later at 1.7 yalms a second: each uses 29016 (159-195) on the member of the party it reaches and is gone (two of the four in each run, 6 s after they came; the party felled the other two 9 s after) |
| 71.5 | Its 29013 (3.7 s) with a helper's 29014 (4.7 s: 829-948 to all four) on one of the party. Again at 144.6 and 247.7 |
| 83.1, 128.6, 172.4, 231.6, 275.8 | **The rows**: ten of base 14593 come where the boss stands; 4.8 s later they leap (29018, then SnapNextMove) to ten places 4 yalms apart on a line through the middle and cast 29019 (3.7 s; 1140 the once it caught a player) across it; 7.7 s after the first leap another row; 7.4 s after that a third leap, and they're gone 22.2 s after coming. The rows were the east-west line (every other one facing the other way), a diagonal (each half its own way), or two half rows crossed: line and diagonal, diagonal and line, diagonal and crossed, diagonal and line, diagonal and crossed. Its 29020 (4.7 s) comes 5.2 and 3.3 s before them or 9.4 s after: the rows keep the fight's clock, not the cast's |
| 119.2, 188.8, 222.2, 292.1 | Four helpers' 29021 (6.7 s: 135-248 a hit) 20 yalms north, south, east and west of the middle; map effects 0-3 (timeline 2, state 4) 1.9 s into the cast |
| 310.3 | **Below 30 %** (28.0 % Connor's, 29.7 % the third run): the last part. Otherwise its 29022, and the question again every 7.6 s: Bob's first run (32.9 %; 30.7 % at 7.6 s) went on 15.5 s later at 29.4 %, his run of 2026-11-18 (32.5 %) 7.1 s later at 29.5 %, as the 29022 and its lock were over. One 29022 in each; here again every other look, a guess |
| the last part | 28628 to the middle, 2.2 s on 29024 (a 70.7 s cast), talks 33223 and 33224 at 4.2 and 11.2 s, map effect 5 (2, 1), weather 106 and talk 33225 at 20.3, talk 33226 at 26.3, the map effect off (8, 4) and weather 2 at 32.3, a helper's 28542 on the party at 32.4, talk 33227 (BNpcName 2664) at 53.3 (and 33228 at 63.4 in Bob's) |
| its health at 0 | (54 and 63 s into 29024.) The cast stopped (ActorControl 15 `[540, 1, 29024, 1]`), the duty complete (the director's 0x40000003, then 0x40000007 `[0, 0]` and `[1, 0]`), the closing cutscene (scene 3 with cutscene 121 and `[1, 18, 0, 0]`), ActionTimeline 73 on it a second later |

**New facts** from the two runs:

* **A battle talk's style byte.** DirectorPopupMessage's byte after the speaker's kind was 0 for the 24 lines under
  BNpcName 2136, **6** for the six under 2183 and **7** for the two under 2664 (both runs). `battle_talk` takes it as
  its seventh argument; what the client draws differently isn't known (Bob's question of 2026-11-17 about portraits
  in battle talk: this is the one field of the packet Kawari had never set).
* **ActorControl 15 (CancelCast) from the fight itself** is `[537, 1, action, 0]` (Kawari's interrupt sends 538), and
  `[540, 1, action, 1]` for a cast its caster dies in. `director:stop_cast` (new) sends the first and drops the
  cast's task, so it never lands.
* **Action effect type 31 with a Knockback row** threw everyone to the arena's edge (29003: row 67, the members
  landing 17 yalms from the middle; 29015: row 13, 22 yalms): `effects:knockback` here, as type 32.
* **The duty's time flags** came as every other duty's (0x40000007 `[0, 1]` 300 s and `[1, 1]` 900 s after the
  start), through the cutscene.
* Duty Support's members were the fourth set: [Party NPCs](party-npcs.md).

**Not known, so guessed or left out** (also in `known-issues.md`): what 29024 does if it lands (9999999 here); what
the four of base 14594 do at the middle (gone here); whether felling the four of base 14590 early changes anything;
the first boss below 22 % before the chain (a 1 HP floor keeps it standing); the second fight past 325.7 s above
30 %; what ActorControls 20 `[325]`, 49 `[16]`, 62 and 70 on the bosses and statuses 3009-3012 do (sent and given as
retail did: **3009 to 3011 are on the first boss from the first, with no timer**, as Kawari Witness saw them when it
first saw the boss on 2026-11-18, and its 28976-28978 gave nothing but their own effect, so the script gives the
three as it brings the boss and those casts do nothing; 3012, from 28984, has no timer either); a wipe (the fight the
party fell in begins again); the damage of what hit no one (28980, 28983, 28997, 29000, 29001; 29010 hit a member
for 1108 in the fourth run). 29023 on a player: 1473 and 1511 to Bob's Paladin, taken as 2238 to anyone else (he
took 1/1.5 of what his other jobs had from this duty's attacks, nine actions compared).

**Tests**: duty fixtures `830-af05916c`, `830-9ba7d981`, `830-50e72631` and `830-6320ae56` (`questkit duty-sim`: 109, 112, 109 and 111 of retail's events; the third's tolerance is 3.5 s for the chain's talk, which retail's boss delayed 3.2 s that time; the
talks, the three's ActionTimelines, every NPC's coming, the completion; and a rehearsed wipe); the fixture's `scenes`
give the script the cutscene's end ([Testing](testing.md)); `a_cast_the_fight_stops_never_lands`
(`bundle_tests.rs`).

## The Praetorium

ContentFinderCondition 16, territory 1044, director 0x80030056, InstanceContent 86, during The Ultimate Weapon (4522).
`content/w1d5_re.lua` (the ContentFinderCondition's ShortCode by analogy with `w1d4_re`, not checked against the
sheet: the server log says if it looked for another name), built on 2026-11-18 from five retail runs with Duty
Support and **unplayed**: Connor's Chronofoil capture af05916c (visit 0.10, a healer), Bob's 9ba7d981 (0.14, left at
the last boss, and 0.16), 50e72631 (0.5) and 6320ae56 (0.3, a Paladin, with Kawari Witness). The five agree on
everything below; the seconds are 6320ae56's from DutyCommence (0x40000001 `[7200]`, 70 s after the opening scene 1,
flags 0x0C53A401). **No layout dump**, as for Castrum Meridianum: what the party sets off by walking is set off by
where they were in the runs (the script's `WHERE_*`).

| When | What |
|---|---|
| 2.1, ~50 | Talk 32920 (BNpcName 1486) ten yalms in from the entrance (2.1 s in three runs, the player already running; 9.7 and 54 s in the others), and 32921 as the last of the four by the entrance falls. Four enemies stand by the entrance (9081944-9081947, bases 14557 and 14558), and BNpcBase 14567 (BNpcName 1486, 258900 HP, layout 9168666), which isn't fought: ActorControl 63 `[48]` moved it, `[11]` cleared a render flag and `[0]` took it away (Witness) |
| 59.7-62.3 | **The lift** (EObj 2012811, GimmickAccessor 0x404): a one-second event, PrepareZoning, the screen's fade (ActorControlSelf 263 `[1]`, 236 `[1]`) and everyone 30 yalms down at (200.1, 156.5, 13.6). Five more enemies below (9081963-9081967, there as the four fall); map effect 1 (timeline 2, state 1) 4.6 s after the lift; the five dead: map effect 2 and ten more (9081969-9081978); map effect 4 further in (north of z -51.5); the ten dead: map effect 3 |
| 343.0-345.6 | **A second lift** (EObj 2012813, GimmickAccessor 0x406) to (240.8, 71.5, 0.3) |
| 355.4 | West of x 208 on that floor: objective 0 done (0x80000015 `[1]`, 0x80000016 `[0, 1]`) and **scene 3 with cutscene 280** `[5, 280, 1, 37, 0]` (flags 0x08502403; 185 s); the first boss is there 2.2 s into it |
| 541.5-653.8 | **The first boss** (9054638, BNpcBase 14450, BNpcName 2134, 62291 HP; arena 15 yalms round (192, 0)): its casts the same to 0.2 s from its pull in every run (`PraetoriumBoss1_14450.json`): 28474 on everyone at 6.2 s, 28467 (no cast: in the middle a second later) at 18.3, 28468 at 20.4 with four helpers' 28469 6 yalms round the middle and four 28470 12 out (a cross the first time; the second, a cross in two runs and turned 30 degrees in two), 28467 and 28471 at 38.6 and 40.7 with a helper's 28472 on each of the party 5.9 s in, then 28474, 28467, 28468, 28473 (a cone), 28467, 28471, 28473 from 58.9 s (every 52.7 s here past the runs). EventLogMessage 2012 `[451, 15]` at its pull (PlaceName 451), EObj 2000789 shown a second later, its wall 2000828 15 s on. 114 to 123 s. It fallen: music 149 (0x80000001), objective 1, talk 32916 a second later, EObjs 2000876, 2000828 and 2000829 gone, 2012815 and 2012819 working, and **the sixteen of the way ahead spawned** (9082001-9082005, 9082022-9082032) |
| 659.1-661.7 | **EObj 2012815** (GimmickAccessor 0x40E): everyone at (146.9, 102.0, -37.5). Talks 32917 and 32918 12.6 and 19.6 s after 32916 |
| 674.8-842.2 | **The armour** (EObj 2012819, GimmickAccessor 0x40A): a one-second event and **Mount 17**; the player uses **action 1128** from it (39 times) (4500 to each enemy it hits, every time) on the sixteen. The last five dead (9082028-9082032): map effect 5, and the way's end (a ramp by (231, 66)) takes everyone below with a PrepareZoning (739.0: (108.7, -106.0, 0.5), still mounted) to seven more (9168161-9168164, 9168175-9168177, there as the party is). A player north of z -29.5: map effect 8 and the first of four of base 14566 (9082092; it dead: map effect 12, timeline 1); the three of 9168162-9168164 dead and a player west of x 54.5: map effect 9 and the second (13); all seven dead and a player west of x -16.5: map effect 10 and the third; it dead: map effect 11 and the fourth 0.6 s on; it dead: map effects 14 and 18 and base 14616 (9089992, 34951 HP) |
| 842.2 | Base 14616 dead: **cutscene 392** `[5, 392, 1, 0, 0]` (107 s), map effect 6 (timeline 1, state 4) with it and 7 (4, 4) 2 s on; the mount goes 1.6 s into it (SetMode 1 and ActorControl 930) |
| 960.3 | West of x -130: objective 2 and **cutscene 283** `[5, 283, 1, 151, 0]` (240 s); the second boss there 2.3 s in |
| 1208.2-1348.2 | **The second boss** (9054847, BNpcBase 14451, BNpcName 2135, 77877 HP; arena 20 yalms round (-164, 0)): talk 32900 4.1 s after its pull in all five runs (map effect 0, timeline 2 state 1, with it), 32902 and 32901 9.2 and 29.6 s after that, 32903 as its health goes below 20 % (19.8 to 19.9 % in four runs, 18.4 in one). Its casts the same to 0.4 s (`PraetoriumBoss2_14451.json`, to 148.8 s: 28483 on its target, 28477 shared, 28482 and 28478 cones, 28476 on everyone, 28481 round it, 28475 with no cast before some of them: where it goes then wasn't read); 0.9 s after each 28479 lands one of base 14452 (9054848) comes, north of the middle then south, with its 28480. EventLogMessage 2012 `[454, 15]`, wall 2000822. 137 to 160 s. It fallen: objective 3, map effects 0 (8, 4) and 15, **cutscene 284** `[5, 284, 1, 149, 0]` (103 s) |
| 1465.9 | **EObj 2012817** (GimmickAccessor 0x408; spawned as cutscene 283 began): objective 4, map effect 16, **cutscene 118** `[5, 118, 0, 152, 1]` (278 s); as it ends everyone at (-550.2, -268.0, 220.3) and the last boss there |
| 1750.7-1995.3 | **The last boss** (9055066, BNpcBase 14453, BNpcName 2136, 93255 HP; a room 15 by 20 round (-562, 220)): talks 32904 (the pull), 32906, 32905, 32907 at 39.5, 62.1 and 80.5 s; [its fight](#the-praetoriums-last-boss) below. It doesn't fall: at 1 HP the duty is complete (0x40000003), **cutscene 374** `[5, 374, 0, 18, 0]`, then everyone out at (-764.7, -344.0, 0.5) |

**A fall on the way** (Bob's, 157 s after the first pull, the members still up): no wipe; he was at the entrance 4 s
later and took the lift again. What a fall at a boss does wasn't seen here.

### The Praetorium's last boss

`PraetoriumBoss3_14453.json` has its own casts (the same to 0.4 s from its pull in four runs): 28495 on everyone at
7.1 s, 28494 on its target, 28493 shared, and the ones the director answers:

* **28496** (no cast, 15.3, 60.9, 79.3 and 109.2 s): it's in the middle a second later.
* **28486** (17.2 s, no cast): a helper's 29779 in the middle (5.7 s) and, 3.1 s on, three of BNpcBase 14456's
  28487 across the middle, 4 yalms apart, all the way the boss faces. A 14456 comes for one cast (1.9 s before it)
  and goes: 28 of them a run.
* **28484** (25.3 and 80.6 s): 3.1 s after it begins the five of BNpcBase 14454 (there with the boss from the first,
  untargetable) go to the room's west edge (28485), 8 yalms apart, at one of two sets of places 4 yalms off each
  other (at random each time); 1.8 s on their 28486, 3.8 s on ActionTimeline 7737, and 4.9 s on five 14456s' 28488
  from those places (a rectangle 40 by 4 each: 1370 to a player).
* **29051** (62.1 and 110.4 s): from 3.9 s in, four rows of three helpers' 29052 2 s apart, north to south or south
  to north (either, in the runs).
* **28491** (116.6 s): 3.9 s in, a helper's 28492 on each of the party.

**The second part** begins 139.7 to 142.9 s in (7.0 s after its 28495 began in two runs, 10.0 in two; its health 11
to 15 %: 141.2 s here): talk 32919; 5.0 s on it can't be targeted (ActorControl 54) and leaves (28496, then 28497
and ActorControls 20 `[2552, 388]`, 62 `[0, 1]` and 63 `[27]`), and **four of BNpcBase 14455** (7715 HP, there from the
first too) take places at the corners of a rectangle 15 by 20 round the middle and fight. Seconds from the talk:

| At | What |
|---|---|
| 7.5 on | The **gauge**: the director's 0x8000000C `[58, n]` once a second, n up 111 a second, **while any of the four is above 1 HP** (it stopped at 65.5 to 76.5 s in the runs, as the last of them got there) |
| 11.8, 34.9, 63.0 | Each of the four still there casts 28498 (4.7 s) on one of the party |
| 16.8, 48.9, 68.0 | The 14454s and the 14456s' 28488 again (the cast 4.2 s after they go to their places) |
| 27.8 | Their 29051, and the rows of 29052 from 31.5 s, south to north in all four runs |
| 49.9 | Their 28491, and the 28492s 3.9 s in |
| | **Each of the four doesn't fall**: at 1 HP it goes (ActorControl 54 `[0]`, ActionTimeline 7737) once it's free: at once, or 2.2 s after the cast it was in (all sixteen in four runs) |
| 67.9 or 77.9 | **It ends**: talk 32908 and ActorControl 70 `[8201036]`. 67.9 s in the one run where all four were at 1 HP by then (66.1 s), 77.9 in the three where the last got there at 68.4, 70.1 and 76.7 s: a look every 10 s here, 87.9 the last whatever they are (a guess: none was missed) |
| +2.1 | Its 28499 on everyone (605 to 1650: what scales it wasn't read) |
| +5.1, +7.4 | ActorControls 70 `[8201619]`, then 62 and 63 `[0]`; the gauge gone (0xFFFFFFFF) at +9.1 |
| +12.2 | Talk 32909, it can be targeted, and fights on: 28495 7.1 s later. It was at 1 HP 7 to 12 s after that cast began |

**What was built for it** (2026-11-18): `director:mount_player(actor_id, mount)` (a player put on a mount by an
object as a quest's event does, `ZoneConnection::mount_by_event`, or taken off it with 0: `FromServer::MountByEvent`);
`info["actor" .. id]` in `onUpdate` (every living NPC by its actor ID too: several from one layout are one entry by
layout); the lifts and the other objects are `onGimmickAccessor` with event action 23 and `warp_players`, as Castrum
Meridianum's device, each working only from its own floor; action 1128 is `PraetoriumArmor_01128.lua` (a set 4500).

**Not known, so guessed or left out**: the event ranges; why the second part began at 139.7 or 142.8 s, and what a
gauge that fills does (28500, an enrage by Boss Mod's name, was never used); where 28475 takes the second boss (it
stays where it is); the bosses' clocks past what the runs show (the rounds go on); **a fall at a boss** (not seen: the
party goes back to where the last lift or cutscene put it, `set_start`, the first two bosses' rooms reset themselves
and the last boss comes back whole with its own; a fall on the way was the entrance on retail, 4 s later, with no
wipe); whether the 14455s attack between their casts (they do); the members during the ride (they walk; retail's
weren't read); whether the server lets a mounted player use 1128 (nothing forbids it, untested); ActorControls 20, 62,
63 and 70 on the last boss (sent as retail did); the ten of BNpcBase 6358 that showed for a moment along the way (left
out); the knockbacks of 28476, 28480, 28481 and 28482 (their rows weren't read: damage only); BNpcBase 14616, which
retail spawned as not hostile, is hostile here so that it can be fought.

**Who's there** (drop-in `territory_1044.json`, from `questkit chronofoil --dropin 1044` over the four captures: 55
battle NPCs of 18 kinds, all `nonpop` but the four by the entrance and base 14567; three helper objects with made-up
layouts 1, 2 and 3 for the bosses' helpers, which retail spawned with layout 0). Timelines `PraetoriumBoss1_14450`,
`PraetoriumBoss2_14451`, `PraetoriumBoss3_14453` and `PraetoriumEnemy_<base>` (fourteen kinds, from the drafts), and
`Helper_9020.json`'s amounts for 28469, 28472, 28492 and 29052.

**Tests**: duty fixtures `16-af05916c`, `16-9ba7d981`, `16-50e72631` and `16-6320ae56` (`questkit duty-sim`: 137, 127,
137 and 137 of retail's events: every talk, objective, NPC brought out, the 14454s' and 14455s' ActionTimelines, the
clear; and a rehearsed wipe at the first two bosses). One known difference (`known.json`): in 6320ae56 one of the
four reached 1 HP 0.3 s before the others' 28491 and went at once; the script's second part runs 1.6 s ahead of that
run's, so there it was already casting.

## At a wipe, duty by duty (2026-11-10)

Every dungeon script has an `onPartyWipe` now (found missing by `questkit duty-sim`'s rehearsed wipes,
[Testing](testing.md#duty-sim)). The server sends the enemies home whole (`reset_after_wipe`); the script forgets
what it kept of a fight under way, so the next pull is a first pull again. None was seen on retail (no wipe in
these duties in Bob's captures, the Stone Vigil's and the Bowl of Embers' apart), and none has been played.

| Duty | What its `onPartyWipe` does |
|---|---|
| Sastasha | The music again at the next pull; Denn's waves taken away, counted again from his pull |
| The Tam-Tara Deepcroft | A room not cleared is pulled anew (music; its boss stays out if its pack had fallen; the leader's line only while it lives); the last boss's adds of the waves reached are taken away, its waves from full health again, its shield off |
| Copperbell Mines | The bosses still alive pulled anew (music), what was due later dropped, the second boss back to its first size |
| The Thousand Maws of Toto-Rak | The adds called and the last boss's tail taken away; the rooms' music and adds and the last boss's adds and tail again (whether it shows its tail again isn't known: its model is left as it was) |
| Haukke Manor | The first boss and the pair pulled anew; the last boss's second add taken away, its calls, teleports and eye wall from the start |
| Brayflox's Longstop | The first boss's waves, the second's add and the goblin fighting it, the third's blobs and the dragon taken away (while their boss lives); music, waves, add, dragon and the last boss's vomits from the start |
| The Navel | (As before: Titan brought back.) The music back to the duty's own |

The Navel's music (four runs: 0fd8bce0, 281f92fb, 48aaf6fe, 650dbfd8): 158 and 160 0.09 s before the first and
second Geocrush, 156 0.09 s before Earthen Fury (`BGM_GEOCRUSH`, `BGM_EARTHEN_FURY` in `s1fa_re.lua`; never set
before).

Sources: [The Stone Vigil (Consolegameswiki)](https://ffxiv.consolegameswiki.com/wiki/The_Stone_Vigil), [The Stone
Vigil dungeon guide (Icy Veins)](https://icy-veins.com/ffxiv/the-stone-vigil-dungeon-guide).
