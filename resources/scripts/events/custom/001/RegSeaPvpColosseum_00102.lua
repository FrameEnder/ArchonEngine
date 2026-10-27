-- Berkoeya Loetahlsyn in Wolves Den Pier

-- Scenes (the client's script has only scene 0: his lines; scene 1, played until 2026-10-27, isn't in it.)
SCENE_00000 = 00000

function onTalk(target, player)
    player:play_scene(SCENE_00000, HIDE_HOTBAR, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end
