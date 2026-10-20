-- Quest battle: Oh Captain, My Captain, for the quest of the same name (SubWil088_00336.)
-- It takes place in a copy of Western Thanalan (territory 273, the same LGBs as 140), so its BNpcs live in the
-- wil_w1f1 drop-in (layer QB_SubWil088_001): Baldewyn fighting beside you (4093947, level 15, 701 HP, a gladiator),
-- Fufulupa (4093955, level 15, 1403 HP, a lancer), a thaumaturge (4093957, level 15, 382 HP) and two Brass Blades
-- (4093961, 4093962, level 14, 175 HP). Layout IDs, levels, HP, positions, display flags and the start position are
-- retail's (Bob's Chronofoil capture of 2026-10-18, `questkit chronofoil --battle 0.23`, `0.25` and `0.27`, territory
-- 273, duty 337: Bob lost the first two runs and won the third.)
--
-- Timed against those runs (seconds from the fight starting, the director's scene 60000, Kawari's onCommence):
--   0      director sequence 2 (sequence 1 is set before the opening cutscene)
--   5      Fufulupa: battle talk 102; 10 Baldewyn: 103
--   Fufulupa at 70% (70.9 -> 70.0, 70.4 -> 69.3, 71.3 -> 68.6): sequence 3 and his 104
--   Fufulupa below 45% (48.2 -> 44.4, 45.7 -> 40.3, 45.5 -> 41.6): sequence 4 and his 105
--   10.2 s after sequence 4 (10.3, 9.8 and 10.2 s; his health then was 31.3%, 30.4% and 24.7%, so a timer): sequence
--          5, half a second after he uses Gut Rip (477) on himself; Baldewyn's 106 3.1 s later. From then on he casts
--          Heartstopper (866) at you, the first 6.1 s after sequence 5 and then every 6 s (three runs alike)
--   everyone but Baldewyn down: 2 s later (the last one's death animation, as retail waits) sequence 6, "Duty
--          Complete", the quest battle's scene 6 (0xF8402EFB) and back out; the quest moves to sequence 255
--   Baldewyn down (both lost runs): his 101 and sequence 0xFE at once, and out 6 s later; the quest stays as it was
-- As on retail (every run): Fufulupa and the thaumaturge go after you, the two Brass Blades after Baldewyn; Baldewyn
-- goes for the thaumaturge first, then the Brass Blades, and only then Fufulupa. He casts Cure (120) on you (at
-- 47-70%) and on himself (at 9-50%), about every 6.5 s between them. How hard each hits is in their timelines'
-- recorded_amounts.

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -238.43, y = 15.2, z = -270.35 }
START_ROTATION = 3.098

BNPC_BALDEWYN = INIT_P_POP_01 or 4093947
BNPC_FUFULUPA = INIT_POP_01 or 4093955
BNPC_MAGE = INIT_POP_02 or 4093957
BNPC_BLADES = { INIT_POP_03 or 4093961, INIT_POP_04 or 4093962 }

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 255
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
-- The scene retail plays after "Duty Complete", and its flags (a cutscene); the player leaves when it ends.
FINISH_SCENE = 6
FINISH_SCENE_FLAGS = 0xF8402EFB
-- How many updates (half a second each) to wait after the player or Baldewyn falls before sending them out.
FAIL_DELAY_UPDATES = 12
-- "Duty Complete" came 2 s after the last enemy's health reached 0, as its death animation ended.
WON_DELAY_UPDATES = 4

ACTION_HEARTSTOPPER = 866
-- Heartstopper's interval once sequence 5 has come (6.0-6.1 s on retail), and the wait for the first, in updates.
HEARTSTOPPER_EVERY = 12

-- Battle talk (DirectorPopupMessage): the speakers' BNpcNames and portraits here, and the content's text IDs, as
-- retail sent them.
SPEAKER_FUFULUPA = { layout = BNPC_FUFULUPA, name = 1381, icon = 0x11e27 }
SPEAKER_BALDEWYN = { layout = BNPC_BALDEWYN, name = 1378, icon = 0x11e26 }
TEXT_BALDEWYN_FELL = 101
-- Steps: { updates after the step before, kind, ... } (Battle:steps)
START_STEPS = {
    { 10, "talk", SPEAKER_FUFULUPA, 102 },
    { 10, "talk", SPEAKER_BALDEWYN, 103 },
}
PHASE_70_STEPS = {
    { 0, "sequence", 3 },
    { 0, "talk", SPEAKER_FUFULUPA, 104 },
}
PHASE_45_STEPS = {
    { 0, "sequence", 4 },
    { 0, "talk", SPEAKER_FUFULUPA, 105 },
    { 20, "sequence", 5 },
    { 0, "call", function()
        heartstopper_in = HEARTSTOPPER_EVERY
    end },
    { 6, "talk", SPEAKER_BALDEWYN, 106 },
}

-- The battle's outline (content/lib/Battle.lua): Baldewyn (a gladiator) is the one the Brass Blades go for, up close,
-- on the thaumaturge first, then the Brass Blades, oldest first, then Fufulupa; he doesn't pull anyone off you.
BATTLE = Battle.new()
ACTION_CURE = 120
BATTLE:ally(BNPC_BALDEWYN, {
    role = "tank",
    protect = false,
    focus = { "mage", "adds", "boss" },
    order = "oldest",
    range = 3,
    heals = {
        { action = ACTION_CURE, targets = { Battle.PLAYER }, below = 70, every = 6 },
        { action = ACTION_CURE, targets = { "self" }, below = 50, every = 6 },
    },
})
BATTLE:enemy(BNPC_FUFULUPA, { group = "boss" })
BATTLE:enemy(BNPC_MAGE, { group = "mage" })
for _, id in ipairs(BNPC_BLADES) do
    BATTLE:enemy(id, { group = "adds", attacks = BNPC_BALDEWYN })
end
BATTLE:onHealth(BNPC_FUFULUPA, 70, function()
    BATTLE:steps(PHASE_70_STEPS)
end)
BATTLE:onHealth(BNPC_FUFULUPA, 45, function()
    BATTLE:steps(PHASE_45_STEPS)
end)

local over = false
local fail_in = nil
local won_in = nil
-- Updates until Fufulupa's next Heartstopper, once sequence 5 has come (nil before.)
heartstopper_in = nil

function onSetup(director)
    director:set_sequence(SEQUENCE_START)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_FIGHTING)
    BATTLE:spawn(director, { BNPC_BALDEWYN })
    BATTLE:spawn(director, { BNPC_FUFULUPA, BNPC_MAGE, BNPC_BLADES[1], BNPC_BLADES[2] })
    BATTLE:steps(START_STEPS)
end

function onUpdate(director, health, player_health, info)
    if fail_in ~= nil then
        fail_in = fail_in - 1
        if fail_in <= 0 then
            fail_in = nil
            director:fail_quest_battle()
        end
        return
    end
    if won_in ~= nil then
        won_in = won_in - 1
        if won_in <= 0 then
            won_in = nil
            director:set_sequence(SEQUENCE_WON)
            director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS)
        end
        return
    end
    if over then
        return
    end
    BATTLE:update(director, health, player_health, info)

    if heartstopper_in ~= nil and BATTLE.alive[BNPC_FUFULUPA] then
        heartstopper_in = heartstopper_in - 1
        if heartstopper_in <= 0 then
            director:use_action(BNPC_FUFULUPA, ACTION_HEARTSTOPPER) -- at the player
            heartstopper_in = HEARTSTOPPER_EVERY
        end
    end
end

function lose(director)
    over = true
    BATTLE:clearSteps()
    heartstopper_in = nil
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    if layout_id == BNPC_BALDEWYN then
        BATTLE:died(director, layout_id)
        director:battle_talk(SPEAKER_BALDEWYN.layout, SPEAKER_BALDEWYN.name, TEXT_BALDEWYN_FELL, SPEAKER_BALDEWYN.icon)
        lose(director)
        return
    end
    BATTLE:died(director, layout_id)
    if BATTLE:enemiesLeft() == 0 then
        over = true
        BATTLE:clearSteps()
        heartstopper_in = nil
        won_in = WON_DELAY_UPDATES
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    lose(director)
end
