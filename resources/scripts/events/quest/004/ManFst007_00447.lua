-- Quest: To Guard a Guardian (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest
-- (questkit replay):
-- Sequence 0:   ACTOR0 (Mother Miounne) offers the quest (scene 0, returns 1 when accepted), then a cutscene (scene 1)
--               gives ITEM0, her message for the Bowlord (UI8BH = 1.)
-- Sequence 1:   ACTOR2 (the Gods' Quiver bow at the door) lets you into the Seat of the First Bow (scene 5, a yes/no;
--               TERRITORYTYPE0 at POPRANGE0.) Hand ITEM0 to ACTOR1 (Lewin) (trade scene 2, 1 when handed over, then
--               cutscene 3): retail sends notice 0 [1, 1], clears the variables and moves to sequence 2.
-- Sequence 2:   ACTOR3 (the Gods' Quiver instructor at the Guardian Tree) offers the quest battle (scene 7); walking into
--               EVENTRANGE0 offers it too (scene 6.) QUESTBATTLE0 takes place in TERRITORYTYPE1, and winning it moves the
--               quest to sequence 3 (content/questbattle/12.lua.)
-- Sequence 3:   back in the Central Shroud, the cutscene after the battle (scene 20) moves to sequence 255.
-- Sequence 255: ACTOR1 (Lewin) and the reward (scene 21.) ACTOR2 lets you in again (scene 22.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_TRADE_OK = 3
SCENE_DOOR = 5
SCENE_RANGE_BATTLE = 6
SCENE_BATTLE = 7
SCENE_AFTER_BATTLE = 20
SCENE_REWARD = 21
SCENE_DOOR_FINISH = 22

NOTICE_DELIVERED = 0

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes (Bob's Chronofoil capture of 2026-10-14)

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_DOOR, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        -- Retail's flags for the reward scene (Bob's Chronofoil capture of 2026-10-16.)
        play(player, SCENE_REWARD, DISABLE_STEALTH | HIDE_HOTBAR | CONDITION_CUTSCENE)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_DOOR_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking up to the Guardian Tree offers the battle too.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_RANGE_BATTLE, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- Back in the Central Shroud after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_AFTER_BATTLE, CUTSCENE_FLAGS)
        return
    end
    player:finish_event()
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
        player:quest_sequence(EVENT_ID, 1)
        setVar(player, 1, true, 1) -- UI8BH: ITEM0
    elseif scene == SCENE_DOOR or scene == SCENE_DOOR_FINISH then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif scene == SCENE_TRADE then
        -- 1 when the message was handed over.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_TRADE_OK then
        player:event_notice(EVENT_ID, NOTICE_DELIVERED, { 1, 1 })
        setVar(player, 1, true, 0)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_BATTLE or scene == SCENE_RANGE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE1)
            return
        end
    elseif scene == SCENE_AFTER_BATTLE then
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
