-- A duty's entrance, its guide (InstanceContentGuide, e.g. 4 for Sastasha's): the client plays the scene itself and
-- the server only ends the event when it returns.
--
-- Retail (Bob's Chronofoil capture of 2026-10-23, Sastasha's entrance): EventStart, then scene 0 with HIDE_HOTBAR |
-- NO_DEFAULT_CAMERA (0x2001), which came back at once with no results; the client then sent ShownActiveHelp 103 and
-- the event finished.

GUIDE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA

function onTalk(target, player)
    player:play_scene(0, GUIDE_FLAGS, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end

function onYield(scene, id, results, player)
    player:finish_event()
end
