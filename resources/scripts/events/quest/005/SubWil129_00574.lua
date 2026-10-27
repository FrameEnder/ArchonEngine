-- Quest: Dressed to Deceive (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/574.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted), then his cutscene (scene 1,
--               retail's flags 0xF8402EFB), which accepts it: RITEM0 and RITEM1, the attire, given as retail gave them
--               (`obtain_item`: the slot, the pop-up, the log line), sequence 1.
-- Sequence 1:   the five to fool, in any order (the client offers each while UI8AL is below 5 and its BitFlag8 is
--               unset: ACTOR1..5, flags 1..5): their scene (2, 5, 8, 11, 14), then, wearing the attire, the next (3, 6,
--               9, 12, 15): its flag, one more done (UI8AL) and notice 0 [done, 5]; the fifth clears the variables and
--               goes to sequence 2. Not wearing both pieces, the one after (4, 7, 10, 13, 16) and nothing more (the
--               client listens for the two items on them: GetListenItems; retail played only the first two, dressed.)
--               ACTOR6 (Thancred) meanwhile: scene 17. ACTOR0, with either piece lost: scene 18, and it's given again.
-- Sequence 2:   ACTOR6 (scene 19, 0x2001, he walks off): notice 1, sequence 3.
-- Sequence 3:   ACTOR7 (Thancred, scene 20), then his cutscene (scene 21, retail's flags 0xF8406EFB): notice 2,
--               sequence 255. ACTOR0, with either piece lost: scene 23, and it's given again.
-- Sequence 255: ACTOR8 (Minfilia, in the Waking Sands) rewards it (scene 24, a yield returning 1.) The attire stays.
-- Scene 22 (Thancred) retail never played and the client offers no one for it.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_THANCRED_WAITING = 17
SCENE_ATTIRE_AGAIN = 18
SCENE_THANCRED = 19
SCENE_THANCRED_AGAIN = 20
SCENE_THANCRED_CUTSCENE = 21
SCENE_ATTIRE_AGAIN_LATER = 23
SCENE_REWARD = 24

-- Retail's flags (Bob's Chronofoil capture b8432613.)
CUTSCENE_FLAGS = 0xF8402EFB
THANCRED_CUTSCENE_FLAGS = 0xF8406EFB
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

-- The five: their first scene, the next dressed and the next not, and their BitFlag8.
MARKS = {
    [ACTOR1] = { scene = 2, flag = 1 },
    [ACTOR2] = { scene = 5, flag = 2 },
    [ACTOR3] = { scene = 8, flag = 3 },
    [ACTOR4] = { scene = 11, flag = 4 },
    [ACTOR5] = { scene = 14, flag = 5 },
}
MARKS_NEEDED = 5
NOTICE_PROGRESS = 0

-- BitFlag8 n is bit 8 - n of UI8F.
function hasFlag(player, flag)
    return player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1)) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- What the inventory says, or `otherwise` if it can't be read (so the quest can't get stuck.)
function ask(player, otherwise, question)
    local ok, answer = pcall(question, player.inventory)
    if not ok or answer == nil then
        return otherwise
    end
    return answer
end

function dressed(player)
    return ask(player, true, function(inventory)
        return inventory:wears(RITEM0) and inventory:wears(RITEM1)
    end)
end

-- The attire the player has lost (in no bag, armoury chest or slot.)
function missingAttire(player)
    local missing = {}
    for _, item in ipairs({ RITEM0, RITEM1 }) do
        if ask(player, 1, function(inventory)
                return inventory:count_owned(item)
            end) == 0 then
            table.insert(missing, item)
        end
    end
    return missing
end

function giveAttire(player, items)
    for _, item in ipairs(items) do
        player:obtain_item(item, 1)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The one being fooled, while their scenes play.
fooling = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local mark = MARKS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and mark ~= nil and (player:get_quest_var(EVENT_ID, 0) & 0x0F) < MARKS_NEEDED
        and not hasFlag(player, mark.flag) then
        fooling = mark
        play(player, mark.scene, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR6 then
        play(player, SCENE_THANCRED_WAITING, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR0 and #missingAttire(player) > 0 then
        play(player, SCENE_ATTIRE_AGAIN, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR6 then
        play(player, SCENE_THANCRED, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR7 then
        play(player, SCENE_THANCRED_AGAIN, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR0 and #missingAttire(player) > 0 then
        play(player, SCENE_ATTIRE_AGAIN_LATER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- One more fooled.
function fooled(player, flag)
    local done = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1 -- UI8AL
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { done, MARKS_NEEDED })
    if done >= MARKS_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 2)
        return
    end
    player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | done)
    setFlag(player, flag)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        giveAttire(player, { RITEM0, RITEM1 })
        player:quest_sequence(EVENT_ID, 1)
    elseif fooling ~= nil and scene == fooling.scene then
        -- Dressed, they're fooled (the next scene); not, they aren't (the one after.)
        if dressed(player) then
            play(player, fooling.scene + 1, HIDE_HOTBAR)
        else
            fooling = nil
            play(player, scene + 2, HIDE_HOTBAR)
        end
        return
    elseif fooling ~= nil and scene == fooling.scene + 1 then
        local flag = fooling.flag
        fooling = nil
        fooled(player, flag)
    elseif scene == SCENE_ATTIRE_AGAIN or scene == SCENE_ATTIRE_AGAIN_LATER then
        giveAttire(player, missingAttire(player))
    elseif scene == SCENE_THANCRED then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_THANCRED_AGAIN then
        play(player, SCENE_THANCRED_CUTSCENE, THANCRED_CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_THANCRED_CUTSCENE then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
