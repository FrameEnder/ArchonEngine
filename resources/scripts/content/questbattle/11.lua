-- Quest battle: Chasing Shadows, for the quest of the same name (ManFst005_00445)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst005_001.)
-- Their layout IDs, health, when each wave arrives and who goes after whom come from Sapphire's script for this battle;
-- the start position and facing from retail (below).
--
-- TODO: The enemies don't use their skills yet, only auto-attacks.

-- Where retail puts the player and which way they face: their SpawnPlayer in Bob's Chronofoil capture (2026-10-14).
START_POSITION = { x = 322.29, y = -1.29, z = -320.7 }
START_ROTATION = -3.137

BNPC_BOSS = 3802315
BNPC_IDA = 3802271
BNPC_PAPALYMO = 3802272

-- The battle's outline (content/lib/Battle.lua). Yda, a pugilist, is there for the boss: up close, and once it's down
-- she finishes what's left, the toughest first. Papalymo, a thaumaturge, is the support: he takes the adds from range
-- (oldest first), stays near you, and heals you when you're low. Neither knows your class, so that's their job
-- whatever you play. The boss and the "B" enemies go after them, the "A" enemies after you.
BATTLE = Battle.new()
-- Roles: both are DPS by class. Yda (a pugilist) plays the tank: she takes the toughest enemy, the boss, on up close
-- (with no tank's tools, she doesn't pull enemies off others.) Papalymo (a thaumaturge) plays a ranged sub-DPS with
-- some support: the adds from range, staying near Yda and you, healing you when you're low.
BATTLE:ally(BNPC_IDA, { role = "tank", class_role = "dps", protect = false, focus = { "boss", "adds" }, order = "highest_hp" })
-- 10 yalms is his timeline's casting distance; how near he keeps to Yda and you is a guess until a recording shows it.
PAPALYMO_RANGE = 10
PAPALYMO_STAYS_WITHIN = 15
INITIAL_ENEMIES = { a = {}, b = { 4094218, 4094220 } }
-- What arrives as the boss's health drops to 75%, 50% and 25%.
WAVES = {
    { health = 75, a = { 4094238, 4094239 }, b = { 4094232, 4094233 } },
    { health = 50, a = { 4094240, 4094471 }, b = { 4094474, 4094475 } },
    { health = 25, a = { 4094472, 4094473 }, b = {} },
}

-- The quest moves to this sequence when the battle is won.
QUEST_SEQUENCE_AFTER_WIN = 2
-- The director's sequence, as retail moves it along (recorded with Kawari Recorder): 1 at the start (the client's script
-- shows the duty objective for it), 2 when the fight gets going 8.5 seconds in, 3, 4 and 5 as each wave arrives, and 6
-- once it's won. Losing sets it to 0xFE (the client shows "Duty Failed"), and the six seconds before sending the player
-- out are what Sapphire's quest battles do.
SEQUENCE_START = 1
SEQUENCE_FIGHTING = 2
SEQUENCE_FIRST_WAVE = 3
SEQUENCE_WON = 6
SEQUENCE_FAILED = 0xFE
-- How many updates (half a second each) after starting the fight gets going.
FIGHT_STARTS_IN = 17
-- How many updates (half a second each) to wait after the player falls before sending them out.
FAIL_DELAY_UPDATES = 12

-- Papalymo heals the player with Cure when they drop below this much health (in percent), at most every few seconds.
ACTION_CURE = 120
HEAL_BELOW_HEALTH = 50
HEAL_COOLDOWN_UPDATES = 8

-- Ida and Papalymo's battle chatter, by key in the quest's text sheet. The server looks the lines up in the game data.
-- (On retail these are the battle talk window, DirectorPopupMessage; until their text IDs are recorded, they're said in
-- /say.) When and in what order
-- is from Kawari Recorder's recording of retail: the first lines 4.5 seconds after the fight gets going, the first
-- wave's 10 seconds after it arrives, each pair 5 seconds apart.
TEXT_PREFIX = "TEXT_MANFST005_00445_"
LINES_START = { { BNPC_PAPALYMO, "PAPALYMO_BATTLETALK_01" }, { BNPC_IDA, "YDA_BATTLETALK_01" } }
LINES_START_DELAY = 9
LINES_WAVE = {
    [1] = { { BNPC_IDA, "YDA_BATTLETALK_02" }, { BNPC_PAPALYMO, "PAPALYMO_BATTLETALK_02" } },
}
LINES_WAVE_DELAY = 19
-- When the last wave arrives, Papalymo yells "Curses! Not now!" (NpcYell 118, a speech bubble) about 6.5 seconds later.
YELL_LAST_WAVE = 118
YELL_LAST_WAVE_DELAY = 13
-- Once the last wave's enemies are defeated, Papalymo says this.
LINE_LAST_WAVE_CLEARED = { { BNPC_PAPALYMO, "PAPALYMO_BATTLETALK_03" } }
-- Said when Papalymo heals the player, at most this often (in updates.)
LINE_HEAL = "PAPALYMO_BATTLETALK_04"
HEAL_LINE_COOLDOWN_UPDATES = 40
-- How many updates apart lines said together are.
LINE_GAP_UPDATES = 10

-- Set once the battle is won or lost.
local over = false
-- Counts down after the player falls, see onUpdate.
local fail_in = nil
local next_wave = 1
-- The last wave's enemies, until they're all defeated.
local last_wave = nil
local heal_line_cooldown = 0

-- Papalymo's role, with his heal (and its line, at most every so often.) He's described after Yda's so the tank is
-- known when he's placed.
BATTLE:ally(BNPC_PAPALYMO, {
    role = "dps",
    class_role = "dps",
    split = false,
    focus = { "adds", "boss" },
    order = "oldest",
    range = PAPALYMO_RANGE,
    keep_near = { anchors = { BNPC_IDA, Battle.PLAYER }, within = PAPALYMO_STAYS_WITHIN },
    heals = { {
        action = ACTION_CURE,
        targets = { Battle.PLAYER },
        below = HEAL_BELOW_HEALTH,
        every = HEAL_COOLDOWN_UPDATES / 2,
        say = function(director)
            if heal_line_cooldown <= 0 then
                BATTLE:steps({ { 0, "say", BNPC_PAPALYMO, TEXT_PREFIX .. LINE_HEAL } })
                heal_line_cooldown = HEAL_LINE_COOLDOWN_UPDATES
            end
        end,
    } },
})
BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_IDA })

-- The "A" enemies go after the player, the "B" ones after Papalymo.
function enemies(wave)
    local ids = {}
    for _, id in ipairs(wave.a) do
        BATTLE:enemy(id, { group = "adds" })
        table.insert(ids, id)
    end
    for _, id in ipairs(wave.b) do
        BATTLE:enemy(id, { group = "adds", attacks = BNPC_PAPALYMO })
        table.insert(ids, id)
    end
    return ids
end

-- Lines said one after another (LINE_GAP_UPDATES apart), the first after `delay` updates.
function lines(list, delay)
    local steps = {}
    for n, line in ipairs(list) do
        local gap = n == 1 and (delay or 0) or LINE_GAP_UPDATES
        if type(line[2]) == "number" then
            table.insert(steps, { gap, "yell", line[1], line[2] })
        else
            table.insert(steps, { gap, "say", line[1], TEXT_PREFIX .. line[2] })
        end
    end
    return steps
end

-- Each wave as the boss's health drops to it: its enemies, the director's sequence, the lines.
for n, wave in ipairs(WAVES) do
    BATTLE:onHealth(BNPC_BOSS, wave.health, function(director)
        if over then
            return
        end
        BATTLE:spawn(director, enemies(wave))
        director:set_sequence(SEQUENCE_FIRST_WAVE + n - 1)
        if LINES_WAVE[n] ~= nil then
            BATTLE:steps(lines(LINES_WAVE[n], LINES_WAVE_DELAY))
        end
        if WAVES[n + 1] == nil then
            BATTLE:steps(lines({ { BNPC_PAPALYMO, YELL_LAST_WAVE } }, YELL_LAST_WAVE_DELAY))
            last_wave = enemies(wave)
            BATTLE:onCleared(last_wave, function()
                BATTLE:steps(lines(LINE_LAST_WAVE_CLEARED))
            end)
        end
    end)
end

function onSetup(director)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_START)
    BATTLE:spawn(director, { BNPC_IDA, BNPC_PAPALYMO })
    BATTLE:spawn(director, { BNPC_BOSS })
    BATTLE:spawn(director, enemies(INITIAL_ENEMIES))
    -- The fight gets going FIGHT_STARTS_IN updates in, and the first lines follow.
    -- (A step runs the update after its count is up, hence the one less.)
    BATTLE:steps({ { FIGHT_STARTS_IN - 1, "sequence", SEQUENCE_FIGHTING } })
    BATTLE:steps(lines(LINES_START, LINES_START_DELAY))
end

-- health: the health (in percent) of every living NPC, by layout ID
-- player_health: the player's health in percent (nil if they're not here)
-- info: everyone's real numbers and position (see Battle:update)
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
    if heal_line_cooldown > 0 then
        heal_line_cooldown = heal_line_cooldown - 1
    end
    BATTLE:update(director, health, player_health, info)
end

function onActorDeath(director, layout_id, position)
    if over then
        return
    end
    BATTLE:died(director, layout_id)
    if BATTLE:enemiesLeft() == 0 and BATTLE.alive[BNPC_BOSS] == nil then
        over = true
        BATTLE:clearSteps()
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
    end
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    BATTLE:clearSteps()
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
