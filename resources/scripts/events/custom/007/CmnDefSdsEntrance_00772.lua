-- A dungeon's entrance offering Duty Support (Sastasha's, in Limsa Lominsa's It's Probably Pirates.)
--
-- Retail (Bob's Chronofoil capture of 2026-10-23): EventStart, then scene 0 with HIDE_HOTBAR | NO_DEFAULT_CAMERA
-- (0x2001). The client's script is the menu: choosing Duty Support sends ClientTrigger 2650 with the duty (DawnContent
-- 200 there), which the server answers (DutySupportContent, main.rs), and the scene comes back; the event finishes and
-- the Duty Support window opens, where registering (ClientTrigger 2654) enters the duty with its members.

ENTRANCE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA

function onTalk(target, player)
    player:play_scene(0, ENTRANCE_FLAGS, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end

function onYield(scene, id, results, player)
    player:finish_event()
end
