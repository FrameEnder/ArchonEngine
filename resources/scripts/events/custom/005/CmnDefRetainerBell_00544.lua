-- Summoning bells (docs/src/fork/retainers.md)
--
-- How retail runs it (Bob's Chronofoil captures, 2026-09-15):
-- * scene 0 (flags 0x2000) is the list of retainers: the client asks for it with client trigger 9003 (command 2);
--   choosing one returns their ID as two numbers (high half first), closing it returns nothing;
-- * the server then calls the retainer and plays scene 2, their menu, with {4, the retainer's actor ID, 3, 0};
-- * closing the menu returns from scene 2: the server sends them away and plays scene 1, the list again, with
--   {2, their ID's two halves}; that returns like scene 0.

-- Scenes
SCENE_LIST = 00000
SCENE_LIST_AGAIN = 00001
SCENE_RETAINER_MENU = 00002
SCENE_00003 = 00003 -- Unknown, but it's mentioned in Scripter
SCENE_NO_APPROVAL = 01000 -- You have not yet received approval to hire retainers.
SCENE_NO_RETAINER = 01001 -- You have not yet hired a retainer.
SCENE_01002 = 01002 -- Unknown, does nothing right now
SCENE_01003 = 01003 -- Unknown, does nothing right now

function onTalk(target, player)
    -- Plays the list, or scene 1001 without retainers.
    player:open_summoning_bell()
end

function onReturn(scene, results, player)
    if (scene == SCENE_LIST or scene == SCENE_LIST_AGAIN) and #results >= 2 then
        -- Plays the retainer's menu.
        player:summon_retainer(results[1], results[2])
    elseif scene == SCENE_RETAINER_MENU then
        -- Plays the list again.
        player:dismiss_retainer()
    else
        player:finish_event()
    end
end
