-- The armoire (CustomTalk 82, CmnDefCabinet).
--
-- Retail (Bob's Chronofoil capture of 2026-10-24, the Hourglass): scene 0 with no parameters opens its menu; the
-- client then asks for the armoire's contents (ClientTrigger 423 and 427) and stores or withdraws items (424, 425),
-- all answered by the server (docs/src/fork/glamour.md), and returns from the scene when the menu closes.

SCENE_MENU = 00000

function onTalk(target, player)
    player:play_scene(SCENE_MENU, HIDE_HOTBAR, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end
