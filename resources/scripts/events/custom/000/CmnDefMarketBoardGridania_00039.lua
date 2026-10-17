-- Marketboard object
--
-- The scene opens the marketboard's window; what it shows comes from the server's marketboard packets while it's open
-- (zone_connection/marketboard.rs, docs/src/fork/marketboard.md.) As retail plays it (Bob's Chronofoil captures of
-- 2026-09-15): scene 0 with HIDE_HOTBAR | NO_DEFAULT_CAMERA and no parameters; it returns when the window closes.

-- Scenes
SCENE_00000 = 00000 -- Show marketboard UI

function onTalk(target, player)
    player:play_scene(SCENE_00000, HIDE_HOTBAR | NO_DEFAULT_CAMERA, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end
