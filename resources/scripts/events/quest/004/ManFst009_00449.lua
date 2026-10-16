-- Quest: Renewing the Covenant (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest
-- (questkit replay):
-- Sequence 0:   ACTOR0 (Mother Miounne) offers the quest (scene 0, returns 1 when accepted), then explains it (scene 1.)
-- Sequence 1:   ACTOR1 (Estaine) at Mih Khetto's Amphitheatre (scene 2.) Wearing RITEM0, the Monoa mask from Festive
--               Endeavors, she takes you to your place (scene 3) and the ceremony plays (cutscene 4), which moves to
--               sequence 255; without it she sends you to put it on (scene 5.) ACTOR2 (Beatin) has a line (scene 6.)
-- Sequence 255: ACTOR4 (the silent conjurer) lets you into the Lotus Stand (scene 8, a yes/no; TERRITORYTYPE0 at
--               POPRANGE1), where ACTOR3 (Kan-E-Senna) gives the reward (scene 7.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ESTAINE = 2
SCENE_WEARING = 3
SCENE_CEREMONY = 4
SCENE_NOT_WEARING = 5
SCENE_BEATIN = 6
SCENE_REWARD = 7
SCENE_DOOR = 8

-- The Lotus Stand's entrance (POPRANGE0 is where the cutscene leaves you.)
LOTUS_STAND_ENTRANCE = POPRANGE1 or 3877982

CUTSCENE_FLAGS = FADE_OUT | CONDITION_CUTSCENE | HIDE_UI

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

-- Whether RITEM0 (the mask) is worn. If the equipment can't be read, assume it is, so the quest can't get stuck.
function wearingMask(player)
    local ok, head = pcall(function()
        return player.inventory.equipped.head.id
    end)
    return not ok or head == nil or head == RITEM0
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ESTAINE, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_BEATIN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_DOOR, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ESTAINE then
        if wearingMask(player) then
            play(player, SCENE_WEARING, HIDE_HOTBAR)
        else
            play(player, SCENE_NOT_WEARING, HIDE_HOTBAR)
        end
        return
    elseif scene == SCENE_WEARING then
        play(player, SCENE_CEREMONY, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_CEREMONY then
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_DOOR then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, LOTUS_STAND_ENTRANCE)
            return
        end
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
