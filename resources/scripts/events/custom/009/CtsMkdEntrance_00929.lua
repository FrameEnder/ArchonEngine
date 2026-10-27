-- Jeffroy at the Phantom Village

-- Scenes (the client's script: 0 turns to the player, then the menu; 1 is the menu alone.) Until 2026-10-27 this
-- played 1, so he never turned to face you.
SCENE_MENU = 00000
SCENE_MENU_AGAIN = 00001

function onTalk(target, player)
    player:play_scene(SCENE_MENU, 0, {0})
end

function onReturn(scene, results, player)
    if scene == SCENE_MENU or scene == SCENE_MENU_AGAIN then
        if #results == 2 then
            -- TODO: why does it pass a help menu as the second param?
            local contentIndex = results[1]
            local content
            if contentIndex == 1 then
                content = CONTENT01
            elseif contentIndex == 2 then
                content = CONTENT02
            elseif contentIndex == 3 then
                content = CONTENT03
            else
                printf(player, "Unknown content "..contentIndex)
            end

            player:register_for_content(content)
        end
    end

    player:finish_event()
end
