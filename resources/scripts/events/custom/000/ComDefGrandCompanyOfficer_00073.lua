-- A Grand Company's personnel officer (CustomTalk 720969, ComDefGrandCompanyOfficer_00073): the storm, serpent and
-- flame personnel officers.
--
-- From the client's script (ComDefGrandCompanyOfficer_00073.luab, Bob's Script Check bundle of 2026-10-27) and Bob's
-- Chronofoil captures 9c4e397d (Limsa Lominsa), bab8c255 (Ul'dah) and 0845b9a2 (Gridania) of 2026-10-29: every scene
-- with flags 0x2000.
-- * Scene 0 greets and returns 1 to a member of the officer's company (the menu follows), 2 to a member of another
--   (changing companies, scene 7, isn't done: the event ends) and 0 to someone in none (it ends.)
-- * Scene 1 is the menu, returning 1 for supply and provisioning missions (PROCUREMENT_MISSION, 720971, played nested:
--   retail's EventStart type 7, its scene 0; the menu again once it ends), 2 for the rank (scene 2), 3 for the
--   officer's explanation (scene 3), 4 to leave. Retail played the menu again after each.
-- * Scene 2 tells the rank and, when the client's own checks for a promotion pass (the rank-up quest, the Hunting Log,
--   the seals: GetGcRankUpCondition), asks for it (GrandCompanyRankUp) and returns 1; otherwise 0 (retail's three
--   times.) On 1 the server promotes (`promote_grand_company`) and plays scene 4, the ceremony.
-- * Scene 4 returns which reward the new rank brings, by the client's IsRankReward: 1 expert delivery (rank 6, scene
--   5: its how-to), 2 the level 30 Grand Company leves (rank 5, scene 6), 3 the level 45 leves (rank 8, scene 9), 4
--   changing companies (rank 9, scene 10), 0 none. What follows the reward scene isn't recorded; the menu plays again.

SCENE_GREETING = 0
SCENE_MENU = 1
SCENE_RANK = 2
SCENE_EXPLAIN = 3
SCENE_PROMOTED = 4

GREETING_MEMBER = 1

MENU_SUPPLY = 1
MENU_RANK = 2
MENU_EXPLAIN = 3

-- The supply and provisioning missions' CustomTalk (the client script's PROCUREMENT_MISSION.)
PROCUREMENT_MISSION = 720971

-- Scene 4's result and the scene that shows that reward.
REWARD_SCENES = { [1] = 5, [2] = 6, [3] = 9, [4] = 10 }

-- Kawari's scene for returning from a nested event (not a real scene): the menu plays again.
SCENE_RETURNED_FROM_NEST = 255
CONTINUES_AFTER_NESTED = true

FLAGS = HIDE_HOTBAR -- 0x2000, retail's

function menu(player)
    player:play_scene(SCENE_MENU, FLAGS, {})
end

function onTalk(target, player)
    player:play_scene(SCENE_GREETING, FLAGS, {})
end

function onReturn(scene, results, player)
    local result = results[1]

    if scene == SCENE_GREETING then
        if result == GREETING_MEMBER then
            menu(player)
            return
        end
    elseif scene == SCENE_MENU then
        if result == MENU_SUPPLY then
            player:start_event(PROCUREMENT_MISSION, EVENT_TYPE_NEST, 0)
            player:play_scene(0, FLAGS, {})
            return
        elseif result == MENU_RANK then
            player:play_scene(SCENE_RANK, FLAGS, {})
            return
        elseif result == MENU_EXPLAIN then
            player:play_scene(SCENE_EXPLAIN, FLAGS, {})
            return
        end
    elseif scene == SCENE_RANK then
        if result == 1 then
            player:promote_grand_company()
            player:play_scene(SCENE_PROMOTED, FLAGS, {})
        else
            menu(player)
        end
        return
    elseif scene == SCENE_PROMOTED then
        local reward = REWARD_SCENES[result]
        if reward ~= nil then
            player:play_scene(reward, FLAGS, {})
        else
            menu(player)
        end
        return
    elseif scene == SCENE_EXPLAIN or scene == SCENE_RETURNED_FROM_NEST or scene == 5 or scene == 6 or scene == 9
        or scene == 10 then
        menu(player)
        return
    end

    player:finish_event()
end
