-- The Ultimate Weapon's last battle (quest 4522: its QUESTBATTLE0 is InstanceContent 5061, not a QuestBattle; director
-- 0x800313C5, territory 1052, duty 867.) A solo duty, entered from the quest and begun as a duty is, like Cape
-- Westwind (5060.lua); won into the quest's sequence 7 and out to the Waking Sands.
--
-- Two retail runs: Connor's Chronofoil capture af05916c (2026-11-09, visit 0.14) and Bob's 9ba7d981 (2026-11-17,
-- visit 0.22; 0.20 is a run left after 29 s.) Their boss's casts, the helpers' casts and spots, the orbs and every
-- talk were the same to within half a second for 527 s, so **the whole fight is one clock from the pull**, and the
-- tables below are that clock, Connor's run's numbers (`questkit chronofoil --battle`; layout IDs, levels, HP and
-- spots are its SpawnNpcs', drop-in `territory_1052.json`.) Both runs ended as his health reached 0, at 528 and 527 s.
--
-- The boss (9161273, base 14643, BNpcName 2143, 119119 HP) auto-attacks and casts from his timeline
-- (`QB5061Boss_14643.json`: the first part from the pull, the second from talk 33704.) Round him:
--   * helpers (base 9020, layout 0; retail had 49) cast where the director puts them (HELPER_CASTS); each 29329
--     leaves lightning where it landed (an event object, LIGHTNING) that goes off as the player comes near (a
--     helper's 29330 from its spot) or goes with the rest; 29335 is cast where the player stands;
--   * nine or eight orbs (base 14644) come five times, cast 29333 7.7 s later and go 22.5 s after coming (ORB_WAVES);
--   * four of base 14697 stand out of sight until they cast 29762 from four spots of the arena's edge, 19.5 yalms
--     from the middle, one of two sets of four (Connor's run: A, A, B, B; Bob's: B, B, B, A: taken at random here);
--   * 29336 takes him away and SnapNextMove brings him out 1.8 s later where his next cast begins (BOSS_MOVES);
--   * an ally (9161275, base 14645, BNpcName 2183) comes as he's pulled and stands aside.
--
-- The middle of the fight is the director's (seconds from the pull):
--   167.1  he can't be targeted. 177.3 his 29754; 177.6 four of base 14699 round the middle; 178.1 ActorControl 63 [6]
--   179.3  the four can be targeted, each tethered to the player (ActorControl 35 [0, 197, player, 15]); a gauge
--          (the director's 0x8000000C [61, n]) climbs from 0 to 10000 by 215.1, 28 a tenth of a second
--   215.1  the four can't be targeted, ActorControl 70 [8201620], talk 33701, his 29787; 216.0 a helper in the middle
--          uses 29788 on the player (Attract 68, status 1614); 218.4 his 29343 (2.2 s); 221.3 his 29780, 63 [0],
--          the gauge gone (0x8000000C [61, 0xFFFFFFFF]), status 2999 on the player, the director's 0x8000001A [1, 1]
--   251.9  a helper's 29565 on the player: 9999999. They fall. 257.2 the music off; 260.7 talk 33713; 279.9 music 81
--   288.1  the ally in the middle: 29344 on the player (status 3123, their HP doubled); talks 33702 and 33703
--   301.75 the player up again, whole (T_PLAYER_KNEELS and the rest, below, are what they go through meanwhile)
--   312.3  status 2999 off, the director's 0x8000001B [1, 0]; 312.7 talk 33704 and his
--          second part; 312.8 ActionTimeline 7747; 315.9 music 165 and he can be targeted
-- His health at 0: he can't be targeted, his cast cancelled (ActorControl 537), the arena's map effect off, the
-- director's scene 4 in a GameComplete event; then the quest to sequence 7 and out to the Waking Sands.
--
-- Not known, so guessed or left out: what happens past 527 s (the second part's points end; nothing follows); what
-- the gauge counts and whether felling the four of base 14699 changes anything (Connor felled one, Bob none: the
-- same clock); what ActorControls 63, 70 and 35, the director's
-- 0x8000001A and 0x8000001B and statuses 661, 662, 1614, 2056, 2999, 3122 and 3123 do (sent and given as retail did);
-- the damage of what never hit anyone (29329, 29333, 29335, 29756, 29760); losing outside the middle (a duty's wipe,
-- as Cape Westwind's.)

START_POSITION = { x = -690.0, y = -185.73, z = 480.0 }
START_ROTATION = -1.571
OPENING_SCENE_FLAGS = 0x0C002001
-- Where retail's player came in at the Waking Sands afterwards (Bob's capture.)
EXIT_AFTER_WIN = { territory = 212, x = -11.07, y = 0.01, z = 0.07, rotation = 1.571 }

BOSS = 9161273
ORB = 9161274
ALLY = 9161275
STATUE = 9277808
SHADE = 9277809
HELPER = 0
-- (Retail had 49 helpers; the most at work at once here is two dozen.)
HELPER_COUNT = 24

FLOOR_Y = -185.66
MIDDLE = { x = -704.0, z = 480.0 }
ALLY_SPOT = { x = -707.0, y = -185.57, z = 465.0, rotation = 0.0 }
-- Where the four of base 14697 wait, and the two sets of spots they cast from (x, z, facing the middle.)
STATUE_REST = { x = -704.34, z = 463.76, rotation = 1.571 }
STATUE_SPOTS = {
    { { -684.5, 480.0, -1.571 }, { -690.24, 466.21, -0.785 }, { -704.01, 460.47, 0.0 }, { -717.8, 466.21, 0.785 } },
    { { -722.01, 472.53, 1.178 }, { -722.04, 487.45, 1.963 }, { -711.48, 498.01, 2.749 }, { -696.56, 498.01, -2.749 } },
}
-- Where he stands as the second part begins.
SECOND_PART_SPOT = { x = -704.0, z = 475.7, rotation = 0.0 }

ACTION_HELPER_RING = 29329
ACTION_HELPER_RING_AFTER = 29330
ACTION_ORB = 29333
ACTION_STATUE = 29762
ACTION_BEGIN_MIDDLE = 29754
ACTION_GAUGE_FULL = 29787
ACTION_KNOCK_DOWN = 29788
ACTION_MIDDLE_CAST = 29343
ACTION_MIDDLE_END = 29780
ACTION_FELL = 29565
ACTION_ALLY = 29344

TIMELINE_STATUE_OUT = 7747
TIMELINE_STATUE_GONE = 7737
TIMELINE_BOSS_BACK = 7747

-- The timeline's anchors (`restart_timeline`): 33701 has no points (the middle), 33704 the second part.
PART_NONE = 33701
PART_SECOND = 33704

CONTROL_63 = 63
CONTROL_70 = 70
CONTROL_TETHER = 35
CONTROL_CANCEL_CAST = 537
TETHER = 197
COMMAND_TEXT_2 = 0x10000002
TEXT_START = 33708
COMMAND_GAUGE = 0x8000000C
GAUGE = 61
GAUGE_FULL = 10000
GAUGE_GONE = 0xFFFFFFFF
COMMAND_MIDDLE_BEGINS = 0x8000001A
COMMAND_MIDDLE_ENDS = 0x8000001B
BGM_NONE = 1
BGM_FALLEN = 81
BGM_SECOND = 165
STATUS_PLAYER_MIDDLE = 2999
MAP_EFFECT_ON = { timeline = 2, state = 1 }
MAP_EFFECT_OFF = { timeline = 8, state = 4 }

-- The clock of what isn't in a table, in seconds from the pull (see the top.)
T_MAP_EFFECT = 10.6
T_UNTARGETABLE = 167.1
T_MIDDLE = 176.0
T_BEGIN_MIDDLE = 177.3
T_SHADES = 177.6
T_CONTROL_63_ON = 178.1
T_SHADES_TARGETABLE = 179.3
T_GAUGE_FULL = 215.1
T_KNOCK_DOWN = 216.0
T_SHADES_GO = 221.86
T_SHADES_FADE = 222.30
T_SHADES_GONE = 223.68
CONTROL_SHADE_GOES = 47
T_MIDDLE_CAST = 218.4
T_MIDDLE_END = 221.3
T_FELL = 251.9
-- The player through the middle, the same in Bob's three runs (Chronofoil captures 9ba7d981, 6320ae56 and 50e72631) to
-- a tenth of a second. Kawari had none of it but 2999 and the getting up, which came at 312.3 (Bob's marks of
-- 2026-11-28: "lahabrea cutscene broken not displaying at all how it should", "warrior of light not displayed at all
-- how they was in retail", "chain vfx still showing around me unlike retail"):
--   221.3  with 2999, ActorControlSelf 263 [1]; 228.2 ActorControlSelf 240 [0]
--   246.95 ActionTimeline 8548 on them; 247.3 status 1614 (the helper's 29788's) off; 248.0 status 2526 (param 6196)
--   251.9  the helper's 29565 fells them (2526 isn't in their statuses after; 2999 is)
--   271.9  status 2397 (param 116), off at 300.7
--   301.75 they're up, whole (the ally's 3123 has doubled their HP by then): the director's log message 9344 [1],
--          status 418 for 6 s, ActorControl 1529 [77], status 2056 (param 396)
--   312.3  2999 off, ActorControlSelf 263 [0], the director's 0x8000001B [1, 0]
-- What 240, 263, 1529 and the statuses do isn't known: sent and given as retail did.
T_PLAYER_240 = 228.2
T_PLAYER_KNEELS = 246.95
T_CHAINS_OFF = 247.3
T_PLAYER_2526 = 248.0
T_PLAYER_2397 = 271.9
T_PLAYER_2397_OFF = 300.7
T_PLAYER_UP = 301.75
CONTROL_PLAYER_263 = 263
CONTROL_PLAYER_240 = 240
CONTROL_TIMELINE = 407
CONTROL_PLAYER_1529 = 1529
TIMELINE_PLAYER_KNEELS = 8548
STATUS_CHAINS = 1614
STATUS_BEFORE_FALL = 2526
STATUS_BEFORE_FALL_PARAM = 6196
STATUS_FALLEN = 2397
STATUS_FALLEN_PARAM = 116
STATUS_UP = 418
STATUS_UP_FOR = 6.0
STATUS_AFTER = 2056
STATUS_AFTER_PARAM = 396
LOG_UP = 9344
-- The director's log messages on the clock (EventLogMessage, no parameters), the same in the three runs.
LOGS = { { 43.2, 10079 }, { 164.45, 10079 }, { 191.77, 10080 }, { 204.3, 10080 } }
T_MUSIC_OFF = 257.2
T_MUSIC_FALLEN = 279.9
T_ALLY = 288.1
T_UP = 312.3
T_SECOND = 312.7
T_BOSS_BACK = 312.8
T_TARGETABLE = 315.9
-- His ActorControl 70s: { seconds from the pull, value } (with NpcYells 13806, 13807, 13806 and 13806, and talk 33701.)
CONTROL_70S = { { 6.04, 8201621 }, { 159.25, 8201622 }, { 215.13, 8201620 }, { 333.98, 8201621 }, { 499.55, 8201621 } }
ORB_CASTS_AFTER = 7.85 -- (7.9 and 7.8 s after they came, Bob's 9ba7d981)
ORB_GONE_AFTER = 22.5
-- What the helpers' 29329 leave (Bob's mark of 2026-11-28: "even simple floor attacks are not being displayed like
-- Dark Thunder"; Kawari had nothing there, and a 29330 from all seven spots 7.7 s after the cast instead.) Retail,
-- both times in each of Bob's three runs (Chronofoil captures 9ba7d981, 6320ae56 and 50e72631), 4.7 to 4.9 s after
-- the casts began (0.7 to 0.9 after they landed): seven SpawnObjects of EObj 2012984, flags byte 5, radius 1, facing
-- as the helpers did, the director their handler, each spot's own second argument, at the spots below (the casts'
-- to a hundredth of a yalm, the floor's height there.) They stayed 19.6 s from the casts' beginning the first time
-- (19.59 to 19.62) and 24.6 the second (24.59 to 24.62): ActorControl 106 [7] from each, and DeleteObject 3.1 to 3.4 s
-- later. The one time a player walked into one (50e72631, 9.6 s after the casts began, 3.6 yalms from its middle in
-- their last position before it and 4.4 in the one before that: 29330's own reach is 4), that one went alone: its
-- 106 [7], a helper put on its spot (ActorSetPos) using 29330 on them (744), and its DeleteObject 3.3 s later.
EOBJ_LIGHTNING = 2012984
EOBJ_LIGHTNING_FLAGS = 5
EOBJ_LIGHTNING_FACING = 1.571
CONTROL_EOBJ_106 = 106
LIGHTNING_AFTER = 4.7
LIGHTNING_REACH = 4.0
LIGHTNING_STAYS = { [16.3] = 19.6, [395.36] = 24.6 } -- by the casts' time in HELPER_CASTS
LIGHTNING_DELETED_AFTER = 3.2
LIGHTNING_SPRUNG_DELETED_AFTER = 3.3
-- { x, y, z, SpawnObject's second argument }
LIGHTNING = {
    { -704.0, -185.66, 480.0, 0x00560003 },
    { -708.0, -185.53, 470.0, 0x00550003 },
    { -700.0, -185.53, 490.0, 0x00540003 },
    { -714.0, -185.53, 476.0, 0x00530003 },
    { -694.0, -185.53, 484.0, 0x00520003 },
    { -711.0, -185.53, 487.0, 0x00510003 },
    { -697.0, -185.53, 473.0, 0x00500003 },
}
live_lightning = {}
sprung_lightning = {}
lightning_leaves = {}
STATUE_OUT_BEFORE = 2.1
STATUE_GONE_AFTER = 6.0

QUEST_SEQUENCE_AFTER_WIN = 7
FINISH_SCENE = 4
FINISH_SCENE_FLAGS = 0x0C002001

-- The helpers' casts: { seconds from the pull, action, x, z, facing }; x nil is where the player stands then.
ACTION_SETS = 29756
SETS_APART = 4.01 -- seconds between the first set of 29756 and the second (4.0 to 4.02 in the runs)
HELPER_CASTS = {
    { 16.3, 29329, -714.02, 475.97, 1.57 },
    { 16.3, 29329, -711.02, 486.99, 1.57 },
    { 16.3, 29329, -708, 469.99, 1.57 },
    { 16.3, 29329, -704.01, 479.97, 1.57 },
    { 16.3, 29329, -700.01, 489.98, 1.57 },
    { 16.3, 29329, -697.02, 472.98, 1.57 },
    { 16.3, 29329, -694.03, 483.97, 1.57 },
    { 68.41, 29335 },
    { 70.47, 29760, -704.01, 479.97, -1.57 },
    { 70.47, 29760, -704.01, 479.97, 3.14 },
    { 70.47, 29760, -704.01, 479.97, 1.57 },
    { 70.47, 29760, -704.01, 479.97, -0 },
    { 70.65, 29335 },
    { 72.84, 29335 },
    { 81.34, 29756, -706.14, 477.87, -2.35 },
    { 81.34, 29756, -706.14, 482.11, -0.79 },
    { 81.34, 29756, -701.9, 477.87, 2.36 },
    { 81.34, 29756, -701.9, 482.11, 0.78 },
    { 83.18, 29756, -710.38, 473.62, -2.35 },
    { 83.18, 29756, -710.38, 486.35, -0.79 },
    { 83.18, 29756, -697.66, 473.62, 2.36 },
    { 83.18, 29756, -697.66, 486.38, 0.78 },
    { 84.96, 29756, -714.66, 469.41, -2.35 },
    { 84.96, 29756, -714.63, 490.59, -0.79 },
    { 84.96, 29756, -693.42, 469.38, 2.36 },
    { 84.96, 29756, -693.42, 490.62, 0.78 },
    { 85.36, 29756, -707.03, 479.97, -1.57 },
    { 85.36, 29756, -704.01, 476.98, 3.14 },
    { 85.36, 29756, -704.01, 482.99, -0 },
    { 85.36, 29756, -701.01, 479.97, 1.57 },
    { 87.15, 29756, -713.01, 479.97, -1.57 },
    { 87.15, 29756, -704.01, 470.97, 3.14 },
    { 87.15, 29756, -704.01, 488.98, -0 },
    { 87.15, 29756, -695, 479.97, 1.57 },
    { 88.95, 29756, -719.02, 479.97, -1.57 },
    { 88.95, 29756, -704.01, 464.99, 3.14 },
    { 88.95, 29756, -704.01, 494.99, -0 },
    { 88.95, 29756, -689.02, 479.97, 1.57 },
    { 113.28, 29758, -704.01, 479.97, -0.23 },
    { 395.36, 29329, -714.02, 475.97, 1.57 },
    { 395.36, 29329, -711.02, 486.99, 1.57 },
    { 395.36, 29329, -708, 469.99, 1.57 },
    { 395.36, 29329, -704.01, 479.97, 1.57 },
    { 395.36, 29329, -700.01, 489.98, 1.57 },
    { 395.36, 29329, -697.02, 472.98, 1.57 },
    { 395.36, 29329, -694.03, 483.97, 1.57 },
    { 427.72, 29335 },
    { 429.77, 29760, -704.01, 479.97, -0.79 },
    { 429.77, 29760, -704.01, 479.97, -2.36 },
    { 429.77, 29760, -704.01, 479.97, 2.36 },
    { 429.77, 29760, -704.01, 479.97, 0.79 },
    { 429.95, 29335 },
    { 432.13, 29335 },
    { 440.66, 29756, -706.14, 477.87, -2.35 },
    { 440.66, 29756, -706.14, 482.11, -0.79 },
    { 440.66, 29756, -701.9, 477.87, 2.36 },
    { 440.66, 29756, -701.9, 482.11, 0.78 },
    { 442.49, 29756, -710.38, 473.62, -2.35 },
    { 442.49, 29756, -710.38, 486.35, -0.79 },
    { 442.49, 29756, -697.66, 473.62, 2.36 },
    { 442.49, 29756, -697.66, 486.38, 0.78 },
    { 444.27, 29756, -714.66, 469.41, -2.35 },
    { 444.27, 29756, -714.63, 490.59, -0.79 },
    { 444.27, 29756, -693.42, 469.38, 2.36 },
    { 444.27, 29756, -693.42, 490.62, 0.78 },
    { 444.67, 29756, -707.03, 479.97, -1.57 },
    { 444.67, 29756, -704.01, 476.98, 3.14 },
    { 444.67, 29756, -704.01, 482.99, -0 },
    { 444.67, 29756, -701.01, 479.97, 1.57 },
    { 446.5, 29756, -713.01, 479.97, -1.57 },
    { 446.5, 29756, -704.01, 470.97, 3.14 },
    { 446.5, 29756, -704.01, 488.98, -0 },
    { 446.5, 29756, -695, 479.97, 1.57 },
    { 448.29, 29756, -719.02, 479.97, -1.57 },
    { 448.29, 29756, -704.01, 464.99, 3.14 },
    { 448.29, 29756, -704.01, 494.99, -0 },
    { 448.29, 29756, -689.02, 479.97, 1.57 },
    { 473.17, 29758, -704.01, 479.97, 2.9 },
}
-- Where he comes out 1.8 s after each 29336 (SnapNextMove): { seconds from the pull, x, z, facing }, where his next
-- cast began.
BOSS_MOVES = {
    { 63.76, -704.01, 479.97, -0 },
    { 112.92, -704.01, 479.97, -0.23 },
    { 169, -704.01, 466.73, -0 },
    { 412.4, -704.01, 492, 3.13 },
    { 422.9, -704.01, 479.97, -0 },
    { 449.58, -716.03, 479.97, 1.57 },
    { 472.86, -704.01, 479.97, 2.9 },
}
-- The orbs: { seconds from the pull they come, their spots (x, z, facing) }. They cast 29333 7.7 s later and go 22.5 s
-- after coming.
ORB_WAVES = {
    { 52.1, { { -715.5, 480, 3.14 }, { -712.14, 471.87, 2.36 }, { -712.14, 488.13, -2.36 }, { -708, 480, 1.57 }, { -704, 491.5, -1.57 }, { -700, 480, 1.57 }, { -695.87, 471.87, 0.79 }, { -695.87, 488.13, -0.79 }, { -692.5, 480, -0 } } },
    { 103.8, { { -715.5, 480, 3.14 }, { -712.14, 471.87, 2.36 }, { -712.14, 488.13, -2.36 }, { -708, 480, 1.57 }, { -704, 491.5, -1.57 }, { -700, 480, 1.57 }, { -695.87, 471.87, 0.79 }, { -695.87, 488.13, -0.79 }, { -692.5, 480, -0 } } },
    { 354, { { -715.5, 480, 3.14 }, { -712.14, 471.87, 2.36 }, { -712.14, 488.13, -2.36 }, { -704, 468.5, 1.57 }, { -704, 491.5, -1.57 }, { -695.87, 471.87, 0.79 }, { -695.87, 488.13, -0.79 }, { -692.5, 480, -0 } } },
    { 463.3, { { -715.5, 480, 3.14 }, { -712.14, 471.87, 2.36 }, { -712.14, 488.13, -2.36 }, { -708, 480, 1.57 }, { -704, 468.5, 1.57 }, { -700, 480, 1.57 }, { -695.87, 471.87, 0.79 }, { -695.87, 488.13, -0.79 }, { -692.5, 480, -0 } } },
    { 517.5, { { -715.5, 480, 3.14 }, { -712.14, 471.87, 2.36 }, { -712.14, 488.13, -2.36 }, { -704, 468.5, 1.57 }, { -704, 491.5, -1.57 }, { -695.87, 471.87, 0.79 }, { -695.87, 488.13, -0.79 }, { -692.5, 480, -0 } } },
}
SHADE_SPOTS = { { -711.08, 472.93, 0.79 }, { -711.08, 487.08, 2.36 }, { -696.93, 472.93, -0.79 }, { -696.93, 487.08, -2.36 } }
-- When the four of base 14697 cast 29762 (seconds from the pull): out 2.1 s before (ActionTimeline 7747), gone 6.0 s
-- after (7737.)
STATUE_CASTS = { 136.8, 363, 407.4, 526.5 }
-- What's said: { seconds from the pull, "talk", text, name, icon, ms } (battle talk) or { seconds, "yell", NpcYell }.
WORDS = {
    { 0, "talk", 33700, 2143, 73174, 6000 },
    { 6.04, "yell", 13806 },
    { 80.23, "talk", 33710, 2143, 73174, 6000 },
    { 129.75, "talk", 33711, 2143, 73174, 6000 },
    { 159.25, "yell", 13807 },
    { 169.85, "talk", 33709, 2143, 73174, 7000 },
    { 215.13, "talk", 33701, 2143, 73174, 5000 },
    { 260.68, "talk", 33713, 2143, 73174, 5000 },
    { 287.89, "talk", 33702, 2183, 0, 6000 },
    { 294.63, "talk", 33703, 2183, 0, 6000 },
    { 312.74, "talk", 33704, 2143, 73174, 7000 },
    { 320.96, "talk", 33705, 2143, 73174, 7000 },
    { 333.98, "yell", 13806 },
    { 344.29, "yell", 13808 },
    { 374.16, "talk", 33706, 2143, 73174, 5000 },
    { 380.18, "talk", 33712, 2143, 73174, 6000 },
    { 387.19, "talk", 33707, 2183, 0, 7000 },
    { 499.55, "yell", 13806 },
}

ticks = 0
updating = false
later = {}
part = 0
helpers = {}
statues = {}
shades = {}
-- Which of them were felled (by their spot, nearest where one fell.)
shades_fallen = {}
next_helper = 0
boss = nil
ally = nil
player_id = nil
player_at = nil
pulled_at = nil

-- Runs fn so many seconds from now (an update is half a second; a call made outside onUpdate waits for the next.)
function after(seconds, fn)
    local updates = math.max(1, math.floor(seconds * 2 + 0.5))
    if not updating then
        updates = updates + 1
    end
    table.insert(later, { at = ticks + updates, fn = fn })
end

function runLater(director)
    local due, waiting = {}, {}
    for _, entry in ipairs(later) do
        if entry.at <= ticks then
            table.insert(due, entry)
        else
            table.insert(waiting, entry)
        end
    end
    later = waiting
    table.sort(due, function(a, b) return a.at < b.at end)
    for _, entry in ipairs(due) do
        entry.fn(director)
    end
end

function bring(director)
    -- (The battle simulator's spawns return nothing: the layout IDs stand in there.)
    boss = director:spawn_bnpc(BOSS) or BOSS
    helpers = {}
    for i = 1, HELPER_COUNT do
        helpers[i] = director:spawn_bnpc(HELPER, MIDDLE.x, FLOOR_Y, MIDDLE.z, 0.0) or HELPER
        director:hold_bnpc(helpers[i])
    end
    statues = {}
    for i = 1, 4 do
        statues[i] = director:spawn_bnpc(STATUE, STATUE_REST.x, FLOOR_Y, STATUE_REST.z, STATUE_REST.rotation) or STATUE
        director:hold_bnpc(statues[i])
        director:set_targetable(statues[i], false)
    end
end

function reset()
    part = 0
    later = {}
    live_orbs = {}
    live_lightning = {}
    sprung_lightning = {}
    lightning_leaves = {}
    next_helper = 0
    shades = {}
    shades_fallen = {}
    ally = nil
    pulled_at = nil
end

function onSetup(director)
    director:set_sequence(1)
    reset()
    bring(director)
end

function onCommence(director)
    director:command(COMMAND_TEXT_2, { TEXT_START })
end

-- The next helper in turn, put at a spot.
function helperAt(director, x, z, facing)
    next_helper = next_helper % #helpers + 1
    local helper = helpers[next_helper]
    director:warp_bnpc(helper, x, FLOOR_Y, z, facing or 0.0)
    return helper
end

function helperCast(director, row)
    local action, x, z, facing = row[2], row[3], row[4], row[5]
    if x == nil then
        -- Where the player stands.
        if player_at == nil then
            return
        end
        x, z, facing = player_at.x, player_at.z, 0.0
    end
    local helper = helperAt(director, x, z, facing)
    director:actor_use_action(helper, action)
    if action == ACTION_HELPER_RING then
        local wave = row[1]
        after(LIGHTNING_AFTER, function(d) lightningComes(d, wave, x, z) end)
        if not lightning_leaves[wave] then
            lightning_leaves[wave] = true
            after(LIGHTNING_STAYS[wave] or 19.6, function(d) lightningLeaves(d, wave) end)
        end
    end
end

-- The lightning a 29329 leaves where it landed (LIGHTNING.)
function lightningComes(director, wave, x, z)
    if part == "over" then
        return
    end
    local spot, nearest = nil, nil
    for _, row in ipairs(LIGHTNING) do
        local d = (row[1] - x) ^ 2 + (row[3] - z) ^ 2
        if nearest == nil or d < nearest then
            spot, nearest = row, d
        end
    end
    local at = { x = spot[1], y = spot[2], z = spot[3] }
    director:spawn_eobj(EOBJ_LIGHTNING, at, spot[4], 0, EOBJ_LIGHTNING_FLAGS, EOBJ_LIGHTNING_FACING)
    table.insert(live_lightning, { at = at, wave = wave })
end

-- Lightning the player comes within 29330's reach of goes off on them, that one alone.
function lightningSprung(director)
    if player_at == nil or part == "over" then
        return
    end
    local left = {}
    for _, entry in ipairs(live_lightning) do
        local at = entry.at
        if (at.x - player_at.x) ^ 2 + (at.z - player_at.z) ^ 2 <= LIGHTNING_REACH ^ 2 then
            director:eobj_control(EOBJ_LIGHTNING, CONTROL_EOBJ_106, { 7 }, at)
            local helper = helperAt(director, at.x, at.z, 0.0)
            director:actor_use_action(helper, ACTION_HELPER_RING_AFTER)
            entry.sprung = true
            after(LIGHTNING_SPRUNG_DELETED_AFTER, function(d)
                -- (Unless the rest have gone since, and it with them.)
                if not entry.deleted then
                    entry.deleted = true
                    d:delete_eobj(EOBJ_LIGHTNING, at)
                end
            end)
            table.insert(sprung_lightning, entry)
        else
            table.insert(left, entry)
        end
    end
    live_lightning = left
end

-- What's left of a volley's lightning goes together.
function lightningLeaves(director, wave)
    local theirs, left = {}, {}
    for _, entry in ipairs(live_lightning) do
        table.insert(entry.wave == wave and theirs or left, entry)
    end
    live_lightning = left
    if #theirs == 0 or part == "over" then
        return
    end
    for _, entry in ipairs(theirs) do
        director:eobj_control(EOBJ_LIGHTNING, CONTROL_EOBJ_106, { 7 }, entry.at)
    end
    after(LIGHTNING_DELETED_AFTER, function(d)
        for _, entry in ipairs(theirs) do
            if not entry.deleted then
                entry.deleted = true
                d:delete_eobj(EOBJ_LIGHTNING, entry.at)
            end
        end
    end)
end

-- None of it outlasts the battle.
function lightningCleared(director)
    local any = false
    for _, list in ipairs({ live_lightning, sprung_lightning }) do
        for _, entry in ipairs(list) do
            if not entry.deleted then
                entry.deleted = true
                any = true
            end
        end
    end
    if any then
        director:delete_eobj(EOBJ_LIGHTNING, true)
    end
    live_lightning = {}
    sprung_lightning = {}
end

-- The orbs fall 5.05 s after their cast begins (ActorControl 14 and 2 `[2]`, and no Defeated: `kill_bnpc` would log
-- one) and fade 8.0 s after (39), every one of the 43 in Bob's 9ba7d981; those still casting as the boss falls have the
-- cast stopped and fall then (15 `[540, 1, 29333, 1]`, 14, 2 `[2]`: the eight of its last wave.)
CONTROL_KILL = 14
CONTROL_MODE = 2
CONTROL_FADE = 39
CAST_STOPPED_DEAD = 540
ORB_FALLS_AFTER_CAST = 5.05
ORB_FADES_AFTER_FALL = 8.0
live_orbs = {}

function orbFalls(director, orb)
    director:actor_control(orb, CONTROL_KILL, {})
    director:actor_control(orb, CONTROL_MODE, { 2 })
end

function orbWave(director, spots)
    local orbs = {}
    for i, spot in ipairs(spots) do
        orbs[i] = director:spawn_bnpc(ORB, spot[1], FLOOR_Y, spot[2], spot[3]) or ORB
        director:hold_bnpc(orbs[i])
    end
    live_orbs = orbs
    after(ORB_CASTS_AFTER, function(d)
        orbs.casting = true
        for _, orb in ipairs(orbs) do
            d:actor_use_action(orb, ACTION_ORB)
        end
    end)
    after(ORB_CASTS_AFTER + ORB_FALLS_AFTER_CAST, function(d)
        for _, orb in ipairs(orbs) do
            orbFalls(d, orb)
        end
        if live_orbs == orbs then
            live_orbs = {}
        end
    end)
    after(ORB_CASTS_AFTER + ORB_FALLS_AFTER_CAST + ORB_FADES_AFTER_FALL, function(d)
        for _, orb in ipairs(orbs) do
            d:actor_control(orb, CONTROL_FADE, {})
        end
    end)
    after(ORB_GONE_AFTER, function(d)
        d:despawn_bnpc(ORB)
    end)
end

-- Each time, as they're put at their spots, they draw and go into battle with no target (ActorControl 0 `[1, 1]`, 4
-- `[1]`, SetTarget naming 0), and 2.04 s after their going timeline out of it again (0 `[0, 1, 1]`, 4 `[0]`, SetTarget
-- 0): all 28 times of the four in Bob's 9ba7d981.
CONTROL_WEAPON = 0
CONTROL_BATTLE = 4
STATUE_OUT_OF_BATTLE_AFTER = 2.04
STATUE_TARGET_GONE_AFTER = 1.0
function statuesCast(director)
    local spots = STATUE_SPOTS[math.random(#STATUE_SPOTS)]
    for i, statue in ipairs(statues) do
        local spot = spots[i]
        director:warp_bnpc(statue, spot[1], FLOOR_Y, spot[2], spot[3])
        director:actor_control(statue, CONTROL_WEAPON, { 1, 1 })
        director:actor_control(statue, CONTROL_BATTLE, { 1 })
        -- (The player their target as they come out, and no one 1.0 s after they've put their weapons away: SetTarget
        -- in Bob's three runs. `aim_bnpc(statue, 0)` named a helper, whose layout ID is 0.)
        director:aim_bnpc(statue, player_id)
        director:play_action_timeline(statue, TIMELINE_STATUE_OUT)
    end
    after(STATUE_OUT_BEFORE + STATUE_GONE_AFTER + STATUE_OUT_OF_BATTLE_AFTER, function(d)
        for _, statue in ipairs(statues) do
            d:actor_control(statue, CONTROL_WEAPON, { 0, 1, 1 })
            d:actor_control(statue, CONTROL_BATTLE, { 0 })
        end
    end)
    after(STATUE_OUT_BEFORE + STATUE_GONE_AFTER + STATUE_OUT_OF_BATTLE_AFTER + STATUE_TARGET_GONE_AFTER, function(d)
        for _, statue in ipairs(statues) do
            d:aim_bnpc(statue, nil)
        end
    end)
    after(STATUE_OUT_BEFORE, function(d)
        for _, statue in ipairs(statues) do
            d:actor_use_action(statue, ACTION_STATUE)
        end
    end)
    after(STATUE_OUT_BEFORE + STATUE_GONE_AFTER, function(d)
        for _, statue in ipairs(statues) do
            d:play_action_timeline(statue, TIMELINE_STATUE_GONE)
        end
    end)
end

-- The gauge of the middle of the fight: retail's climbed 28 a tenth of a second to 10000.
function gauge(director, began)
    if part ~= "middle" then
        return
    end
    local value = math.min(GAUGE_FULL, math.floor((ticks - began) * 0.5 * 280))
    director:command(COMMAND_GAUGE, { GAUGE, value })
    if value < GAUGE_FULL then
        after(0.5, function(d) gauge(d, began) end)
    end
end

function middle(director)
    part = "middle"
    director:hold_wipe(true)
    director:restart_timeline(BOSS, PART_NONE)
    after(T_BEGIN_MIDDLE - T_MIDDLE, function(d) d:actor_use_action(boss, ACTION_BEGIN_MIDDLE, boss) end)
    after(T_SHADES - T_MIDDLE, function(d)
        shades = {}
        for i, spot in ipairs(SHADE_SPOTS) do
            shades[i] = d:spawn_bnpc(SHADE, spot[1], FLOOR_Y, spot[2], spot[3]) or SHADE
            d:hold_bnpc(shades[i])
            d:set_targetable(shades[i], false)
        end
    end)
    after(T_CONTROL_63_ON - T_MIDDLE, function(d) d:bnpc_control(BOSS, CONTROL_63, { 6 }) end)
    after(T_SHADES_TARGETABLE - T_MIDDLE, function(d)
        for _, shade in ipairs(shades) do
            d:set_targetable(shade, true)
            if player_id ~= nil then
                d:bnpc_control(shade, CONTROL_TETHER, { 0, TETHER, player_id, 15 })
            end
        end
        local began = ticks
        gauge(d, began)
    end)
    after(T_GAUGE_FULL - T_MIDDLE, function(d)
        for _, shade in ipairs(shades) do
            d:set_targetable(shade, false)
        end
        d:actor_use_action(boss, ACTION_GAUGE_FULL, boss)
    end)
    after(T_KNOCK_DOWN - T_MIDDLE, function(d)
        d:actor_use_action(helperAt(d, MIDDLE.x, MIDDLE.z, 0.0), ACTION_KNOCK_DOWN, player_id)
    end)
    after(T_MIDDLE_CAST - T_MIDDLE, function(d) d:actor_use_action(boss, ACTION_MIDDLE_CAST, boss) end)
    -- Those still standing go as the corners of the Porta Decumana do: ActorControl 47, FadeOut (607 `[itself, 1, 0,
    -- 100]`) 0.44 s later and gone 1.38 s after that (the three Bob left, 9ba7d981); one felled before goes as the
    -- fallen do.
    after(T_SHADES_GO - T_MIDDLE, function(d)
        for i, shade in ipairs(shades) do
            if not shades_fallen[i] then
                d:actor_control(shade, CONTROL_SHADE_GOES, {})
            end
        end
    end)
    after(T_SHADES_FADE - T_MIDDLE, function(d)
        for i, shade in ipairs(shades) do
            if not shades_fallen[i] then
                d:fade_out_bnpc(shade)
            end
        end
    end)
    after(T_SHADES_GONE - T_MIDDLE, function(d)
        for i, shade in ipairs(shades) do
            if not shades_fallen[i] then
                d:despawn_bnpc(shade)
            end
        end
        shades = {}
        shades_fallen = {}
    end)
    after(T_MIDDLE_END - T_MIDDLE, function(d)
        part = "falling"
        d:actor_use_action(boss, ACTION_MIDDLE_END)
        d:bnpc_control(BOSS, CONTROL_63, { 0 })
        d:command(COMMAND_GAUGE, { GAUGE, GAUGE_GONE })
        if player_id ~= nil then
            d:gain_effect(player_id, STATUS_PLAYER_MIDDLE, 0, 0)
            d:actor_control(player_id, CONTROL_PLAYER_263, { 1 })
        end
        d:command(COMMAND_MIDDLE_BEGINS, { 1, 1 })
    end)
    local function toPlayer(at, fn)
        after(at - T_MIDDLE, function(d)
            if player_id ~= nil then
                fn(d, player_id)
            end
        end)
    end
    toPlayer(T_PLAYER_240, function(d, player) d:actor_control(player, CONTROL_PLAYER_240, { 0 }) end)
    toPlayer(T_PLAYER_KNEELS, function(d, player)
        d:actor_control(player, CONTROL_TIMELINE, { TIMELINE_PLAYER_KNEELS })
    end)
    toPlayer(T_CHAINS_OFF, function(d, player) d:lose_effect(player, STATUS_CHAINS) end)
    toPlayer(T_PLAYER_2526, function(d, player)
        d:gain_effect(player, STATUS_BEFORE_FALL, STATUS_BEFORE_FALL_PARAM, 0)
    end)
    after(T_FELL - T_MIDDLE, function(d)
        d:actor_use_action(helperAt(d, MIDDLE.x, MIDDLE.z, 0.0), ACTION_FELL, player_id)
        if player_id ~= nil then
            d:lose_effect(player_id, STATUS_BEFORE_FALL)
        end
    end)
    toPlayer(T_PLAYER_2397, function(d, player) d:gain_effect(player, STATUS_FALLEN, STATUS_FALLEN_PARAM, 0) end)
    toPlayer(T_PLAYER_2397_OFF, function(d, player) d:lose_effect(player, STATUS_FALLEN) end)
    after(T_PLAYER_UP - T_MIDDLE, function(d)
        d:log_message(LOG_UP, { 1 })
        d:revive_players()
        if player_id ~= nil then
            d:gain_effect(player_id, STATUS_UP, 0, STATUS_UP_FOR)
            d:actor_control(player_id, CONTROL_PLAYER_1529, { 77 })
            d:gain_effect(player_id, STATUS_AFTER, STATUS_AFTER_PARAM, 0)
        end
    end)
    after(T_MUSIC_OFF - T_MIDDLE, function(d) d:set_bgm(BGM_NONE) end)
    after(T_MUSIC_FALLEN - T_MIDDLE, function(d) d:set_bgm(BGM_FALLEN) end)
    after(T_ALLY - T_MIDDLE, function(d)
        if ally ~= nil then
            d:warp_bnpc(ally, MIDDLE.x, FLOOR_Y, MIDDLE.z, 0.0)
            d:actor_use_action(ally, ACTION_ALLY, player_id)
        end
    end)
    after(T_UP - T_MIDDLE, function(d)
        -- (Should they not be up yet.)
        d:revive_players()
        if player_id ~= nil then
            d:lose_effect(player_id, STATUS_PLAYER_MIDDLE)
            d:actor_control(player_id, CONTROL_PLAYER_263, { 0 })
        end
        d:command(COMMAND_MIDDLE_ENDS, { 1, 0 })
    end)
    after(T_SECOND - T_MIDDLE, function(d)
        part = 2
        d:restart_timeline(BOSS, PART_SECOND)
    end)
    after(T_BOSS_BACK - T_MIDDLE, function(d)
        d:jump_bnpc(BOSS, SECOND_PART_SPOT.x, FLOOR_Y, SECOND_PART_SPOT.z, SECOND_PART_SPOT.rotation)
        d:play_action_timeline(BOSS, TIMELINE_BOSS_BACK)
    end)
    after(T_TARGETABLE - T_MIDDLE, function(d)
        d:set_bgm(BGM_SECOND)
        d:set_targetable(BOSS, true)
        d:hold_wipe(false)
    end)
end

-- The pull: everything on the fight's clock is set going.
function pull(director)
    part = 1
    pulled_at = ticks
    ally = director:spawn_bnpc(ALLY, ALLY_SPOT.x, ALLY_SPOT.y, ALLY_SPOT.z, ALLY_SPOT.rotation) or ALLY
    director:hold_bnpc(ally)
    for _, word in ipairs(WORDS) do
        if word[1] < 0.25 then
            say(director, word)
        else
            after(word[1], function(d) say(d, word) end)
        end
    end
    for _, control in ipairs(CONTROL_70S) do
        after(control[1], function(d) d:bnpc_control(BOSS, CONTROL_70, { control[2] }) end)
    end
    after(T_MAP_EFFECT, function(d) d:map_effect(0, MAP_EFFECT_ON.timeline, MAP_EFFECT_ON.state) end)
    -- The helpers' 29756 come as two sets of twelve 4.0 s apart, the diagonals and the cross, twice in the fight. The
    -- first time the diagonals came first in all four runs; the second time in two of three, the cross first in
    -- Bob's of 2026-11-18 (capture 6320ae56): either at random then.
    local of_29756, cross_first = 0, false
    for _, row in ipairs(HELPER_CASTS) do
        local at = row[1]
        if row[2] == ACTION_SETS then
            local occurrence, set = of_29756 // 24, (of_29756 % 24) // 12
            if of_29756 == 24 then
                cross_first = math.random(2) == 2
            end
            if occurrence == 1 and cross_first then
                at = at + (set == 0 and SETS_APART or -SETS_APART)
            end
            of_29756 = of_29756 + 1
        end
        after(at, function(d) helperCast(d, row) end)
    end
    for _, log in ipairs(LOGS) do
        after(log[1], function(d)
            if part ~= "over" then
                d:log_message(log[2], {})
            end
        end)
    end
    for _, move in ipairs(BOSS_MOVES) do
        after(move[1], function(d) d:jump_bnpc(BOSS, move[2], FLOOR_Y, move[3], move[4]) end)
    end
    for _, wave in ipairs(ORB_WAVES) do
        after(wave[1], function(d) orbWave(d, wave[2]) end)
    end
    for _, at in ipairs(STATUE_CASTS) do
        after(at - STATUE_OUT_BEFORE, statuesCast)
    end
    after(T_UNTARGETABLE, function(d) d:set_targetable(BOSS, false) end)
    after(T_MIDDLE, middle)
end

function say(director, word)
    if part == "over" then
        return
    end
    if word[2] == "talk" then
        director:battle_talk(word[4] == 2183 and ALLY or BOSS, word[4], word[3], word[5], word[6])
    else
        director:yell(BOSS, word[3])
    end
end

function onUpdate(director, health, player_health, info)
    ticks = ticks + 1
    updating = true
    runLater(director)
    watch(director, health, info)
    updating = false
end

function watch(director, health, info)
    if info == nil then
        return
    end
    if info.player ~= nil then
        player_id = info.player.id
        player_at = { x = info.player.x, y = info.player.y, z = info.player.z }
        lightningSprung(director)
    end
    local him = info[BOSS]
    if him == nil then
        return
    end
    -- The pull: he has someone to go for, or has been hit.
    local left = health[BOSS]
    if part == 0 and (him.target ~= nil or (left ~= nil and left < 100)) then
        pull(director)
    end
end

function clear(director)
    lightningCleared(director)
    director:despawn_bnpc(ORB)
    director:despawn_bnpc(SHADE)
    for _, helper in ipairs(helpers) do
        director:despawn_bnpc(helper)
    end
    for _, statue in ipairs(statues) do
        director:despawn_bnpc(statue)
    end
    if ally ~= nil then
        director:despawn_bnpc(ally)
    end
    helpers = {}
    statues = {}
end

function onActorDeath(director, layout_id, position)
    if layout_id == SHADE and position ~= nil then
        local nearest, best = nil, math.huge
        for i, spot in ipairs(SHADE_SPOTS) do
            local dx, dz = spot[1] - position.x, spot[2] - position.z
            if not shades_fallen[i] and dx * dx + dz * dz < best then
                nearest, best = i, dx * dx + dz * dz
            end
        end
        if nearest ~= nil then
            shades_fallen[nearest] = true
        end
    end
    if layout_id == BOSS and part ~= "over" then
        part = "over"
        later = {}
        director:hold_wipe(false)
        director:set_targetable(BOSS, false)
        if boss ~= nil then
            director:bnpc_control(BOSS, CONTROL_CANCEL_CAST, { boss })
        end
        for _, orb in ipairs(live_orbs) do
            if live_orbs.casting then
                director:stop_cast(orb, CAST_STOPPED_DEAD)
            end
            orbFalls(director, orb)
        end
        live_orbs = {}
        director:map_effect(0, MAP_EFFECT_OFF.timeline, MAP_EFFECT_OFF.state)
        clear(director)
        boss = nil
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS, nil, false)
    end
end

-- The player down outside the middle of the fight: a duty's wipe sends them back to the start; everyone goes and
-- comes back as at the beginning.
function onPartyWipe(director)
    director:despawn_bnpc(BOSS)
    director:map_effect(0, MAP_EFFECT_OFF.timeline, MAP_EFFECT_OFF.state)
    clear(director)
    reset()
    bring(director)
end
