-- Retainer vocates: hiring and letting go of retainers (docs/src/fork/retainers.md)
--
-- How retail runs it (Bob's Chronofoil capture of 2026-10-18, game 2026.10.06):
-- * scene 4, then scene 0 (the menu), which yields 37 (answered with 0) and returns the choice;
-- * before each choice's scene, the retainer list is sent with the choice in RetainerInfoEnd;
-- * hiring is scene 1: yield 3 (zeros), 24 (0), 25 with 1 (the appearance editor opens: condition
--   CreatingCharacter), the client's appearance packet, 25 with 0 (editor closed), 36 (2), then the name as an event
--   text answer (yield 5): the retainer is made, RetainerState (hired), answered 0. Scene 1 returns 11: the list again;
-- * letting go is scene 2: yield 13 (0), yield 9 with the retainer's ID: [1, 0] released, [0, 1] refused while they
--   have items on the market;
-- * every visit ends with ActorControlSelf 152 (how many retainers) before EventFinish.

CONDITION = 7 -- OccupiedInQuestEvent, as retail (the default, written out)

-- Scenes
SCENE_MENU = 00000
SCENE_HIRE = 00001
SCENE_RELEASE = 00002
SCENE_00003 = 00003 -- You cannot hire a retainer
SCENE_GREETING = 00004

-- The menu's choices and the scenes retail played for them. Retail also had choice 5 (scene 7, answered with nine
-- numbers on a resume packet Kawari doesn't know) and 8 (scenes 9 and 10); they end the visit here for now.
CHOICE_SCENES = {
    [1] = SCENE_HIRE,
    [3] = SCENE_RELEASE,
}
-- What scene 1 returns when a retainer was hired: the list is sent once more with it.
HIRED = 11

-- Yields
YIELD_MENU = 37
YIELD_HIRE_START = 3
YIELD_HIRE_STEP = 24
YIELD_EDITOR = 25
YIELD_BEFORE_NAME = 36
YIELD_NAME = 5
YIELD_RELEASE_START = 13
YIELD_RELEASE = 9

CONDITION_CREATING_CHARACTER = 31

function onTalk(target, player)
    player:play_scene(SCENE_GREETING, HIDE_HOTBAR, {})
end

function onYield(scene, id, results, player)
    if id == YIELD_MENU or id == YIELD_HIRE_STEP or id == YIELD_RELEASE_START then
        player:resume_event(scene, id, {0})
    elseif id == YIELD_HIRE_START then
        -- Retail answered with eight zeros on a resume opcode (329) that isn't upstream's EventResume8 (513); a
        -- shorter answer with the same value is sent instead.
        player:resume_event(scene, id, {0})
    elseif id == YIELD_EDITOR then
        -- 1: the appearance editor opens, 0: it closed.
        player:set_condition(CONDITION_CREATING_CHARACTER, results[1] == 1)
        player:resume_event(scene, id, {1})
    elseif id == YIELD_BEFORE_NAME then
        player:resume_event(scene, id, {2})
    elseif id == YIELD_NAME then
        player:hire_retainer_at_vocate(scene, id)
    elseif id == YIELD_RELEASE and #results >= 2 then
        player:release_retainer_at_vocate(results[1], results[2], scene, id)
    else
        player:resume_event(scene, id, {0})
    end
end

function finish(player)
    player:send_retainer_count()
    player:finish_event()
end

function onReturn(scene, results, player)
    if scene == SCENE_GREETING then
        player:play_scene(SCENE_MENU, HIDE_HOTBAR, {})
    elseif scene == SCENE_MENU then
        local choice = results[1] or 0
        local next_scene = CHOICE_SCENES[choice]
        if next_scene ~= nil then
            player:send_retainer_list_at_vocate(choice)
            player:play_scene(next_scene, HIDE_HOTBAR, {})
        else
            finish(player)
        end
    elseif scene == SCENE_HIRE and results[1] == HIRED then
        player:send_retainer_list_at_vocate(HIRED)
        finish(player)
    else
        finish(player)
    end
end
