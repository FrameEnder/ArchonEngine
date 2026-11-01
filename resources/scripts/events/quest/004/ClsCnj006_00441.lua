-- Quest: In Nature's Embrace
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- df6f5af9 (2026-11-01, the whole quest and its battle, entered three times: its fixture is
-- tools/questkit/fixtures/441.json):
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene (scene
--               1, 0xF8402EFB, a yield too): sequence 1.
-- Sequences 1, 3, 5: a patch of corrupted soil (EOBJECT0, 1, 2): its scene (2, 10, 18: empty), EVENT_ACTION_SEARCH's
--               bar, then the next scene (3, 11, 19: the client's pop message), all 0x2001, which brings out a tainted
--               earth sprite (ENEMY0, 1, 2: layer QB_ClsCnj006_001 of the South Shroud drop-in, at retail's positions,
--               level and HP; timeline TaintedEarthSprite_294), the player's own. Its death: notice 0, 2, 4 and the
--               next sequence. ACTOR1, 2, 3 (Sylphie, where each patch is) meanwhile: scene 8, 16, 24 (inferred: the
--               scene before each one's own, a line of hers; not seen on retail).
-- Sequences 2, 4, 6: ACTOR1, 2, 3 talk and walk off (scene 9, 17, 25, 0x2001): notice 1, 3, 5 and the next sequence.
-- Sequence 7:   ACTOR4 (Sylphie again) asks whether to enter QUESTBATTLE0 (scene 26: YesNoQuestBattle, a yield, 1 for
--               yes), which takes the player to TERRITORYTYPE0 (235). Winning sets sequence 255: that's the battle's
--               director (content/questbattle/17.lua), not this script; retail lost twice first and was asked again.
-- Sequence 255: ACTOR0 rewards it (scene 29, a cutscene and a yield: QuestReward, 1 and the optional item chosen.)
-- The client's scenes 4-7, 12-15, 20-23 (empty, or another pop message) and 27-28 (empty) weren't seen on retail
-- and aren't played.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ENTER_BATTLE = 26
SCENE_REWARD = 29

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
EVENT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- The three patches and the Sylphie by each: in the sequence the patch is searched in, its scene, the scene after
-- the search bar, the sprite it brings out and the notice its death sends; then, a sequence later, her scene and
-- notice. Her line while the sprite's still to be found is `before`.
STEPS = {
    { sequence = 1, soil = EOBJECT0, scene = 2, searched = 3, enemy = ENEMY0, killed = 0,
      sylphie = ACTOR1, before = 8, talk = 9, talked = 1 },
    { sequence = 3, soil = EOBJECT1, scene = 10, searched = 11, enemy = ENEMY1, killed = 2,
      sylphie = ACTOR2, before = 16, talk = 17, talked = 3 },
    { sequence = 5, soil = EOBJECT2, scene = 18, searched = 19, enemy = ENEMY2, killed = 4,
      sylphie = ACTOR3, before = 24, talk = 25, talked = 5 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_step = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_step = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 7 and BASE_ID == ACTOR4 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
        return
    end
    for _, step in ipairs(STEPS) do
        if sequence == step.sequence and BASE_ID == step.soil then
            current_step = step
            play(player, step.scene, EVENT_FLAGS)
            return
        elseif sequence == step.sequence and BASE_ID == step.sylphie then
            play(player, step.before, EVENT_FLAGS)
            return
        elseif sequence == step.sequence + 1 and BASE_ID == step.sylphie then
            current_step = step
            play(player, step.talk, EVENT_FLAGS)
            return
        end
    end
    player:finish_event()
end

-- The search bar on a patch finished: the scene after it.
function onEventActionCast(player)
    local step = current_step
    if step == nil or player:get_quest_sequence(EVENT_ID) ~= step.sequence then
        current_step = nil
        player:finish_event()
        return
    end
    play(player, step.searched, EVENT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    for _, step in ipairs(STEPS) do
        if sequence == step.sequence and layout_id == step.enemy then
            player:event_notice(EVENT_ID, step.killed, {})
            player:quest_sequence(EVENT_ID, step.sequence + 1)
            return
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local step = current_step

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif step ~= nil and scene == step.scene then
        -- Searching the patch: the bar, then the scene after it (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif step ~= nil and scene == step.searched then
        current_step = nil
        -- One still out isn't brought out again.
        player:spawn_quest_bnpc(step.enemy)
    elseif step ~= nil and scene == step.talk then
        current_step = nil
        player:event_notice(EVENT_ID, step.talked, {})
        player:quest_sequence(EVENT_ID, step.sequence + 2)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
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
