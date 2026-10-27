-- Quest: For the Children (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/621.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted), then scene 1 (the client
--               accepts it there): sequence 1.
-- Sequence 1:   ACTOR1 (the uncombed urchin, scene 2): notice 0, sequence 2.
-- Sequence 2:   walking into EVENTRANGE0 (offered while UI8AL is below 2) brings out ENEMY0 and ENEMY1, a zombie and a
--               zombie mage (layout IDs, owned by the player, naming the range as their trigger), with scene 7, 0x2001
--               (the client's pop message, which it skips when they're already out for you). Each kill counts in UI8AL,
--               with no notice. ACTOR6 (Ourcen): scene 8 (0x2001; the client's "won't talk" message while UI8AL is
--               below 2); after both kills, scene 8, then EVENT_ACTION_RESCUE_UNDER_MIDDLE's bar, then scene 9 (0x2001,
--               he walks off): notice 1 [1, 1], sequence 255, the variables cleared. ACTOR1 meanwhile: scene 10.
-- Sequence 255: ACTOR0's reward (scene 17, a yield: 1 and the item chosen.)
-- The client's other scenes (3-6, 11-16: the other urchins, the enemies and EOBJECT0) are empty and retail played none
-- of them. Its BitFlag8 1 check on ACTOR6 retail never set.
-- The two enemies are layer QB_SubWil113_001 of the Eastern Thanalan drop-in, at retail's positions, level and HP.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_URCHIN = 2
SCENE_AMBUSH = 7
SCENE_OURCEN = 8
SCENE_RESCUED = 9
SCENE_URCHIN_WAITING = 10
SCENE_REWARD = 17

-- Retail's flags (Bob's Chronofoil capture b8432613.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

NOTICE_URCHIN = 0
NOTICE_RESCUED = 1
ENEMIES = { [ENEMY0] = true, [ENEMY1] = true }
KILLS_NEEDED = 2

-- UI8AL: the enemies killed.
function kills(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F
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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_URCHIN, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR6 then
        play(player, SCENE_OURCEN, OBJECT_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_URCHIN_WAITING, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 springs the ambush (one of each at a time: nothing new while they're out.)
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or kills(player) >= KILLS_NEEDED then
        player:finish_event()
        return
    end
    player:spawn_quest_bnpc(ENEMY0)
    player:spawn_quest_bnpc(ENEMY1)
    play(player, SCENE_AMBUSH, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if not ENEMIES[layout_id] or player:get_quest_sequence(EVENT_ID) ~= 2 or kills(player) >= KILLS_NEEDED then
        return
    end
    local value = player:get_quest_var(EVENT_ID, 0)
    player:set_quest_var(EVENT_ID, 0, (value & 0xF0) | (kills(player) + 1))
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_RESCUED, OBJECT_FLAGS)
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
    elseif scene == SCENE_URCHIN then
        player:event_notice(EVENT_ID, NOTICE_URCHIN, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_OURCEN then
        if kills(player) >= KILLS_NEEDED then
            -- The rescue's bar; scene 9 plays once it finishes (see onEventActionCast.)
            player:event_action(EVENT_ACTION_RESCUE_UNDER_MIDDLE)
            return
        end
    elseif scene == SCENE_RESCUED then
        player:event_notice(EVENT_ID, NOTICE_RESCUED, { 1, 1 })
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
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
