-- Side talk: what a quest's NPCs say when talked to outside the quest's steps. Loaded by the server after a quest's
-- own script when `resources/data/quest_side_talks.json` has entries for it (`questkit sidetalk` writes them from the
-- client's quest script and the Quest sheet's listeners; see docs/src/fork/quests.md, Side talk).
--
-- The quest's script is asked first. When it does nothing with the talk but end it, and the client's script has a
-- line for this NPC in this sequence, the line is played: a recording has the steps a player took, so a transcribed
-- script answers those, and retail's NPCs have something to say in between as well (Bob, 2026-11-14: "it may not
-- break the flow, but that's no excuse to ignore retail behavior if we can implement it").
--
-- QUEST_SIDE_TALKS (set by the server): a list of { actor = ENpc base, scenes = { ["<sequence>"] = line } },
-- sequence 0 being before the quest is accepted. A line is a scene, played with HIDE_HOTBAR, as retail played the
-- same kind of talk in quest 466 (Bob's Chronofoil captures); or, for the ones read by hand
-- (tools/questkit/sidetalk_hand.json), { scene, flags, after, var, mask }: other flags than HIDE_HOTBAR, and a scene
-- for after something is done (an NPC saluted), played once the quest's variable `var` has a bit of `mask` set.

local script_onTalk = onTalk
local script_onReturn = onReturn

-- The line being said.
local side_scene = nil

-- The scene and flags of this NPC's line in the quest's sequence, or nil.
local function line_for(player)
    local sequence = player:get_quest_sequence(EVENT_ID) or 0
    for _, entry in ipairs(QUEST_SIDE_TALKS or {}) do
        if entry.actor == BASE_ID then
            local line = entry.scenes[tostring(sequence)]
            if type(line) ~= "table" then
                return line, HIDE_HOTBAR
            end
            local scene = line.scene
            if line.after ~= nil and (player:get_quest_var(EVENT_ID, line.var) & line.mask) ~= 0 then
                scene = line.after
            end
            return scene, line.flags or HIDE_HOTBAR
        end
    end
    return nil
end

function onTalk(target, player)
    side_scene = nil
    if script_onTalk ~= nil then
        script_onTalk(target, player)
    end
    local scene, flags = line_for(player)
    if scene ~= nil and player:event_unanswered() then
        side_scene = scene
        player:play_scene(scene, flags, {})
    elseif script_onTalk == nil then
        player:finish_event()
    end
end

function onReturn(scene, results, player)
    if side_scene ~= nil and scene == side_scene then
        side_scene = nil
        player:finish_event()
        return
    end
    if script_onReturn ~= nil then
        script_onReturn(scene, results, player)
    end
end
