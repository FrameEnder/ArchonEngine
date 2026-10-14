-- Quest battle: Chasing Shadows, for the quest of the same name (ManFst005_00445)
-- It takes place in a copy of the Central Shroud, so its BNpcs live in the fst_f1 drop-in (layer QB_ManFst005_001.)
-- Their layout IDs, health, the start position, when each wave arrives and who goes after whom come from Sapphire's
-- script for this battle.
--
-- TODO: The enemies don't use their skills yet, only auto-attacks.

START_POSITION = { x = 323.0, y = -1.28, z = -320.0 }
START_ROTATION = 3.0

BNPC_BOSS = 3802315
BNPC_IDA = 3802271
BNPC_PAPALYMO = 3802272

-- Ida takes on the boss and Papalymo the adds. The boss and the "B" enemies go after them, the "A" enemies after you.
-- How much enemies hate their first target. Damage adds to this, so hitting an enemy a few times pulls it to you.
STARTING_ENMITY = 300
-- How much more Ida and Papalymo hate each new target they're given, so they switch to it and stay on it.
ALLY_ENMITY = 10000
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
-- (On retail these are battle talk pop-ups; until that packet is known, they're said in /say.) When and in what order
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

-- Enemies still standing, by layout ID, and the order they arrived in.
local remaining = {}
local arrived = {}
local next_wave = 1
-- Set once the battle is won or lost.
local over = false
-- Counts down after the player falls, see onUpdate.
local fail_in = nil
-- Counts down until Papalymo can heal again.
local heal_cooldown = 0
local heal_line_cooldown = 0
-- Lines waiting to be said: { updates left, layout ID, key (a text key to say, or an NpcYell ID to yell) }.
local pending_lines = {}
-- Counts up once the fight starts, see onUpdate.
local fight_starts_in = nil
-- The last wave's enemies, until they're all defeated.
local last_wave = nil

-- Queues lines to be said one after another, the first after `delay` updates.
function sayLines(lines, delay)
    delay = delay or 0
    for _, line in ipairs(pending_lines) do
        delay = math.max(delay, line[1] + LINE_GAP_UPDATES)
    end
    for _, line in ipairs(lines) do
        table.insert(pending_lines, { delay, line[1], line[2] })
        delay = delay + LINE_GAP_UPDATES
    end
end

function sayPendingLines(director)
    local still_pending = {}
    for _, line in ipairs(pending_lines) do
        if line[1] <= 0 then
            if type(line[3]) == "number" then
                director:yell(line[2], line[3])
            else
                director:say(line[2], TEXT_PREFIX .. line[3])
            end
        else
            table.insert(still_pending, { line[1] - 1, line[2], line[3] })
        end
    end
    pending_lines = still_pending
end

function spawnEnemy(director, id)
    remaining[id] = true
    table.insert(arrived, id)
    director:spawn_bnpc(id)
end

function spawnEnemies(director, enemies)
    for _, id in ipairs(enemies.a) do
        spawnEnemy(director, id)
        director:set_hate(id, nil, 1) -- the player
    end
    for _, id in ipairs(enemies.b) do
        spawnEnemy(director, id)
        director:set_hate(id, BNPC_PAPALYMO, STARTING_ENMITY)
    end
end

-- The oldest enemy still standing, leaving out the boss if asked.
function oldestRemaining(skip_boss)
    for _, id in ipairs(arrived) do
        if remaining[id] and not (skip_boss and id == BNPC_BOSS) then
            return id
        end
    end
    return nil
end

-- Ida goes for the boss and Papalymo for the adds, oldest first. Either takes whatever's left once theirs are gone.
local ally_priority = 0
function alliesAttack(director)
    local ida_target = nil
    if remaining[BNPC_BOSS] then
        ida_target = BNPC_BOSS
    else
        ida_target = oldestRemaining(false)
    end
    local papalymo_target = oldestRemaining(true) or ida_target

    ally_priority = ally_priority + ALLY_ENMITY
    if ida_target ~= nil then
        director:set_hate(BNPC_IDA, ida_target, ally_priority)
    end
    if papalymo_target ~= nil then
        director:set_hate(BNPC_PAPALYMO, papalymo_target, ally_priority)
    end
end

function onSetup(director)
end

function onCommence(director)
    director:set_sequence(SEQUENCE_START)
    fight_starts_in = FIGHT_STARTS_IN

    director:spawn_bnpc(BNPC_IDA)
    director:spawn_bnpc(BNPC_PAPALYMO)

    spawnEnemy(director, BNPC_BOSS)
    director:set_hate(BNPC_BOSS, BNPC_IDA, STARTING_ENMITY)
    spawnEnemies(director, INITIAL_ENEMIES)

    alliesAttack(director)
end

-- health: the health (in percent) of every living NPC, by layout ID
-- player_health: the player's health in percent (nil if they're not here)
function onUpdate(director, health, player_health)
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

    if fight_starts_in ~= nil then
        fight_starts_in = fight_starts_in - 1
        if fight_starts_in <= 0 then
            fight_starts_in = nil
            director:set_sequence(SEQUENCE_FIGHTING)
            sayLines(LINES_START, LINES_START_DELAY)
        end
    end

    sayPendingLines(director)
    if heal_line_cooldown > 0 then
        heal_line_cooldown = heal_line_cooldown - 1
    end

    if heal_cooldown > 0 then
        heal_cooldown = heal_cooldown - 1
    elseif player_health ~= nil and player_health > 0 and player_health < HEAL_BELOW_HEALTH
        and health[BNPC_PAPALYMO] ~= nil then
        director:use_action(BNPC_PAPALYMO, ACTION_CURE) -- on the player
        heal_cooldown = HEAL_COOLDOWN_UPDATES
        if heal_line_cooldown <= 0 then
            sayLines({ { BNPC_PAPALYMO, LINE_HEAL } })
            heal_line_cooldown = HEAL_LINE_COOLDOWN_UPDATES
        end
    end

    local boss_health = health[BNPC_BOSS]
    if boss_health == nil then
        return
    end

    while WAVES[next_wave] ~= nil and boss_health <= WAVES[next_wave].health do
        spawnEnemies(director, WAVES[next_wave])
        director:set_sequence(SEQUENCE_FIRST_WAVE + next_wave - 1)
        if LINES_WAVE[next_wave] ~= nil then
            sayLines(LINES_WAVE[next_wave], LINES_WAVE_DELAY)
        end
        if WAVES[next_wave + 1] == nil then
            sayLines({ { BNPC_PAPALYMO, YELL_LAST_WAVE } }, YELL_LAST_WAVE_DELAY)
            last_wave = {}
            for _, id in ipairs(WAVES[next_wave].a) do last_wave[id] = true end
            for _, id in ipairs(WAVES[next_wave].b) do last_wave[id] = true end
        end
        next_wave = next_wave + 1
        -- Papalymo turns to the new adds if he'd run out.
        alliesAttack(director)
    end
end

function onActorDeath(director, layout_id, position)
    if not remaining[layout_id] then
        return
    end
    remaining[layout_id] = nil

    if over then
        return
    end

    if next(remaining) == nil then
        over = true
        director:set_sequence(SEQUENCE_WON)
        director:complete_quest_battle(QUEST_SEQUENCE_AFTER_WIN)
        return
    end

    if last_wave ~= nil and last_wave[layout_id] then
        last_wave[layout_id] = nil
        if next(last_wave) == nil then
            last_wave = nil
            sayLines(LINE_LAST_WAVE_CLEARED)
        end
    end

    -- Whoever the allies were fighting may be gone, so point them at the next one.
    alliesAttack(director)
end

-- Losing sends you back to where you entered, still on the same step of the quest, so you can try again.
function onPlayerDeath(director)
    if over then
        return
    end
    over = true
    director:set_sequence(SEQUENCE_FAILED)
    fail_in = FAIL_DELAY_UPDATES
end
