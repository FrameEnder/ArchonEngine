-- Hakeel Ja in Tulliyollal

-- Scenes
SCENE_00000 = 00000 -- His greeting, then a menu whose answers depend on the Dawntrail story quests (the client's
                     -- script has only this scene.)

function onTalk(target, player)
    player:play_scene(SCENE_00000, HIDE_HOTBAR, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end
