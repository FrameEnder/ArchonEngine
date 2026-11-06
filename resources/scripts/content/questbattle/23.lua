-- Quest battle 23: The Rematch (ClsGla300_00265, the Gladiators' level 30 quest.)
-- It takes place in territory 257, a copy of Eastern Thanalan (retail's positions in it stand on Eastern Thanalan's
-- ground, to the hundredth of a yalm, against the same capture's visits to 145), so its NPCs live in the wil_w1f3
-- drop-in (layer QB_ClsGla300_003). Levels, HP, positions, gear, display and character data flags and the start spot
-- are retail's (Bob's Chronofoil capture accc33a9 of 2026-11-06, `questkit chronofoil --battle 0.15`: one run, won; a
-- gladiator synced to 34). Everyone is out from the start (43 of them):
--   * along the way up from the start, in pairs, each taking the player on as they come close: a lancer (base 658),
--     two conjurers (659), a pugilist (660), two archers (661) and two thaumaturges (662);
--   * at the camp above, waiting for sequence 5: four pugilists (664), a lancer (663) and a conjurer (687) against
--     the three on your side (4011016, base 665, BNpcName 689, the one who speaks; 4011017, 666; 4011018, 667);
--   * two champions (4011120, base 657, BNpcName 690, yours; 4011122, base 1324, theirs; level 50, 258900 HP each),
--     who fight each other from sequence 5 on and never fall;
--   * twenty soldiers fighting in pairs in the background from the start (bases 1335 and 1336 yours, 1338-1340
--     theirs; level 15, 191400 HP each, chipping 15-40 at a time: none falls.)
-- Then the last wave: a thaumaturge on you (4011442, base 656) and three more lancers (663) on the one who speaks.
--
-- Timed against the run (seconds from the fight starting, the director's scene 60000):
--   0       sequence 2; the background soldiers start (1.0-2.0 s)
--   5.1     battle talk 145 (your champion, no portrait)
--   the player at the third pair (20.1; within 6 yalms of them): sequence 3
--   the player at the archers below the camp (36.2; within 9 yalms): sequence 5 and talk 146 (the one who speaks);
--           the camp and the champions fight
--   the conjurer down (94.0): its talk 152 at once; heard 2.4 s later: sequence 6, the last wave 0.35 s after
--   talk 147 10.05 s after sequence 6
--   every one of them down (177.6, at once: 0.05 s): "Duty Complete" (60001) and sequence 8, then the battle's scene 8
--           (a cutscene, 0xF8402EFB) 3.1 s later; the quest moves to 255 with every variable 0 (its two key items
--           gone)
-- The one who speaks Cures himself and the other two below 40 % (retail's at 6-36 % before the heal, every 6.9 s),
-- the third gladiator him and the second below 45 % (31-44 %, every 7.6 s), the conjurer its side below 30 %
-- (0-29 %, every 5.9 s.)

-- Where retail puts the player and which way they face: their SpawnPlayer in the capture.
START_POSITION = { x = -14.92, y = -31.0, z = -32.94 }
START_ROTATION = 1.586
-- Retail's opening scene 1 here (Kawari's default too, written out.)
OPENING_SCENE_FLAGS = 0xF841AEFB

-- On your side.
BNPC_SPEAKER = 4011016
BNPC_SECOND = 4011017
BNPC_THIRD = 4011018
CAMP_ALLIES = { BNPC_SPEAKER, BNPC_SECOND, BNPC_THIRD }
BNPC_CHAMPION = 4011120
BNPC_RIVAL = 4011122

-- Along the way: pairs that take the player on as they come close.
PATH = { 4011007, 4011008, 4011009, 4011010, 4011011, 4011012, 4011014, 4011015 }
-- Where sequences 3 and 5 came (retail's positions of the third pair and of the archers below the camp), and how
-- close the player was then.
THIRD_PAIR = { { x = 0.7, y = -31.0, z = -7.2 }, { x = 10.1, y = -31.0, z = -7.1 } }
THIRD_PAIR_YALMS = 6
ARCHERS = { { x = 4.6, y = -17.5, z = 20.2 }, { x = 6.4, y = -17.5, z = 20.2 } }
ARCHERS_YALMS = 9

-- The camp: each one and whom it goes for at sequence 5, as retail's did.
BNPC_CONJURER = 4011022
CAMP = {
    { 4011021, BNPC_SPEAKER }, { 4011027, BNPC_SECOND }, { 4011028, BNPC_SECOND }, { 4011029, BNPC_THIRD },
    { 4011030, BNPC_THIRD }, { BNPC_CONJURER, BNPC_SPEAKER },
}
-- The last wave.
LAST_WAVE = { { 4011419, BNPC_SPEAKER }, { 4011422, BNPC_SPEAKER }, { 4011426, BNPC_SPEAKER }, { 4011442, nil } }

-- The background soldiers, yours and theirs in pairs, as retail's fought.
SOLDIERS = {
    { 4011234, 4011235 }, { 4011237, 4011236 }, { 4011241, 4011242 }, { 4011244, 4011243 }, { 4011246, 4011247 },
    { 4011249, 4011248 }, { 4011260, 4011261 }, { 4011262, 4011263 }, { 4011266, 4011265 }, { 4011267, 4011268 },
}

QUEST_SEQUENCE_AFTER_WIN = 255
-- The two key items (UI8BH and UI8BL) gone with the win, as retail's UpdateQuest.
QUEST_VARS_AFTER_WIN = { [1] = 0 }
SEQUENCE_FIGHTING = 2
SEQUENCE_THIRD_PAIR = 3
SEQUENCE_CAMP = 5
SEQUENCE_LAST_WAVE = 6
SEQUENCE_WON = 8
SEQUENCE_FAILED = 0xFE
FINISH_SCENE = 8
FINISH_SCENE_FLAGS = 0xF8402EFB
FAIL_DELAY_UPDATES = 12

-- Timers, in updates (half a second each.)
DEATH_NOTICED_AFTER = 4
WAVE_OUT_AFTER = 1
TALK_START_AFTER = 10
TALK_147_AFTER = 20
-- A timer started from onActorDeath counts the update it was started in, so one more.
FROM_DEATH = 1

-- The battle talk (the content's text IDs, retail's.)
SPEAKER_CHAMPION = { layout = BNPC_CHAMPION, name = 690, icon = 0 }
SPEAKER_ALLY = { layout = BNPC_SPEAKER, name = 689, icon = 0 }
SPEAKER_CONJURER = { layout = BNPC_CONJURER, name = 708, icon = 0 }
TALK_START = 145
TALK_CAMP = 146
TALK_CONJURER_DOWN = 152
TALK_LAST_WAVE = 147

ACTION_CURE = 120

BATTLE = Battle.new()
BATTLE:ally(BNPC_SPEAKER, {
    role = "dps",
    split = false,
    focus = { "camp", "last" },
    order = "nearest",
    heals = { { action = ACTION_CURE, targets = { BNPC_SPEAKER, BNPC_SECOND, BNPC_THIRD }, below = 40, every = 6.9 } },
})
BATTLE:ally(BNPC_SECOND, { role = "dps", split = false, focus = { "camp", "last" }, order = "nearest" })
BATTLE:ally(BNPC_THIRD, {
    role = "dps",
    split = false,
    focus = { "camp", "last" },
    order = "nearest",
    heals = { { action = ACTION_CURE, targets = { BNPC_SPEAKER, BNPC_SECOND }, below = 45, every = 7.6 } },
})
BATTLE:ally(BNPC_CHAMPION, { role = "dps", split = false, focus = { "rival" } })
BATTLE:enemy(BNPC_RIVAL, { group = "rival", attacks = BNPC_CHAMPION, passive = true, unaware = true })
for _, layout in ipairs(PATH) do
    BATTLE:enemy(layout, { group = { "path", "foes" }, passive = true, unaware = true })
end
for _, enemy in ipairs(CAMP) do
    local role = { group = { "camp", "foes" }, attacks = enemy[2], passive = true, unaware = true }
    if enemy[1] == BNPC_CONJURER then
        role.heals = { { action = ACTION_CURE, targets = "allies", below = 30, every = 5.9 } }
    end
    BATTLE:enemy(enemy[1], role)
end
for _, enemy in ipairs(LAST_WAVE) do
    BATTLE:enemy(enemy[1], { group = { "last", "foes" }, attacks = enemy[2] })
end
for i, pair in ipairs(SOLDIERS) do
    BATTLE:ally(pair[1], { split = false, focus = { "soldier" .. i } })
    BATTLE:enemy(pair[2], { group = "soldier" .. i, attacks = pair[1], passive = true, unaware = true })
end

local over = false
local fail_in = nil
local phase = 1
local last_wave_out = false
-- The ones that count (the way, the camp, the last wave) out and not yet down.
local standing = {}
local timers = {}

function after(updates, fn)
    table.insert(timers, { updates, fn })
end

function runTimers(director)
    local current = timers
    timers = {}
    for _, timer in ipairs(current) do
        timer[1] = timer[1] - 1
        if timer[1] <= 0 then
            timer[2](director)
        else
            table.insert(timers, timer)
        end
    end
end

function talk(director, speaker, text)
    director:battle_talk(speaker.layout, speaker.name, text, speaker.icon, nil, speaker.kind)
end

function layoutsOf(list)
    local layouts = {}
    for _, entry in ipairs(list) do
        table.insert(layouts, type(entry) == "table" and entry[1] or entry)
    end
    return layouts
end

function playerNear(info, points, yalms)
    local player = info and info.player
    if player == nil then
        return false
    end
    for _, point in ipairs(points) do
        local dx, dy, dz = point.x - player.x, point.y - player.y, point.z - player.z
        if dx * dx + dy * dy + dz * dz <= yalms * yalms then
            return true
        end
    end
    return false
end

-- Everyone out but the last wave, before the opening cutscene, as retail's 39: no one fights yet.
function onSetup(director)
    director:set_sequence(1)
    local everyone = { BNPC_CHAMPION, BNPC_RIVAL }
    for _, list in ipairs({ CAMP_ALLIES, PATH, layoutsOf(CAMP) }) do
        for _, layout in ipairs(list) do
            table.insert(everyone, layout)
        end
    end
    for _, pair in ipairs(SOLDIERS) do
        table.insert(everyone, pair[1])
        table.insert(everyone, pair[2])
    end
    local allies = { BNPC_CHAMPION }
    for _, layout in ipairs(CAMP_ALLIES) do
        table.insert(allies, layout)
    end
    for _, pair in ipairs(SOLDIERS) do
        table.insert(allies, pair[1])
    end
    BATTLE:holdBack(allies)
    BATTLE:spawn(director, everyone)
    for _, layout in ipairs(PATH) do
        standing[layout] = true
    end
    for _, layout in ipairs(layoutsOf(CAMP)) do
        standing[layout] = true
    end
end

-- The fight starts: the background soldiers go at each other, and the ones along the way notice the player.
function onCommence(director)
    phase = SEQUENCE_FIGHTING
    director:set_sequence(SEQUENCE_FIGHTING)
    local soldiers = {}
    for _, pair in ipairs(SOLDIERS) do
        table.insert(soldiers, pair[1])
        BATTLE:engage(director, { pair[2] }, pair[1])
    end
    BATTLE:release(director, soldiers)
    for _, layout in ipairs(PATH) do
        director:set_notice(layout, true)
    end
    after(TALK_START_AFTER, function(d) talk(d, SPEAKER_CHAMPION, TALK_START) end)
end

-- The camp and the champions fight.
function camp(director)
    phase = SEQUENCE_CAMP
    director:set_sequence(SEQUENCE_CAMP)
    talk(director, SPEAKER_ALLY, TALK_CAMP)
    for _, enemy in ipairs(CAMP) do
        BATTLE:engage(director, { enemy[1] }, enemy[2])
    end
    BATTLE:engage(director, { BNPC_RIVAL }, BNPC_CHAMPION)
    BATTLE:release(director, { BNPC_CHAMPION, BNPC_SPEAKER, BNPC_SECOND, BNPC_THIRD })
end

function lastWave(director)
    phase = SEQUENCE_LAST_WAVE
    director:set_sequence(SEQUENCE_LAST_WAVE)
    after(WAVE_OUT_AFTER, function(d)
        for _, layout in ipairs(layoutsOf(LAST_WAVE)) do
            standing[layout] = true
        end
        last_wave_out = true
        BATTLE:spawn(d, layoutsOf(LAST_WAVE))
    end)
    after(TALK_147_AFTER, function(d) talk(d, SPEAKER_ALLY, TALK_LAST_WAVE) end)
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
    if over then
        return
    end
    runTimers(director)
    if over then
        return
    end
    if phase == SEQUENCE_FIGHTING and playerNear(info, THIRD_PAIR, THIRD_PAIR_YALMS) then
        phase = SEQUENCE_THIRD_PAIR
        director:set_sequence(SEQUENCE_THIRD_PAIR)
    end
    if phase < SEQUENCE_CAMP and phase >= SEQUENCE_FIGHTING and playerNear(info, ARCHERS, ARCHERS_YALMS) then
        camp(director)
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if not standing[layout_id] then
        return
    end
    standing[layout_id] = nil
    if layout_id == BNPC_CONJURER then
        -- Its last words at once, as its health reached 0; the last wave once its death has played.
        talk(director, SPEAKER_CONJURER, TALK_CONJURER_DOWN)
        after(DEATH_NOTICED_AFTER + FROM_DEATH, function(d)
            if not over and phase < SEQUENCE_LAST_WAVE then
                lastWave(d)
            end
        end)
    end
    -- The win came with the last one's health at 0 on retail (0.05 s after), not after its death animation.
    if last_wave_out and next(standing) == nil then
        over = true
        timers = {}
        phase = SEQUENCE_WON
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN, false, FINISH_SCENE, FINISH_SCENE_FLAGS,
            QUEST_VARS_AFTER_WIN)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    timers = {}
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
