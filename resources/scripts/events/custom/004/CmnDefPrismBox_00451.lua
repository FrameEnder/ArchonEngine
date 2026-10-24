-- The glamour dresser (CustomTalk 451, CmnDefPrismBox).
--
-- Retail (Bob's Chronofoil capture of 2026-10-24, the Hourglass): scene 0 with no parameters opens its menu; the
-- client then asks for the dresser (ClientTrigger 2350) and the plates (2356), stores and restores items (2351, 2352)
-- and saves plates (SaveGlamourPlate), all answered by the server (docs/src/fork/glamour.md), and returns from the
-- scene when the menu closes.

SCENE_MENU = 00000

function onTalk(target, player)
    player:play_scene(SCENE_MENU, HIDE_HOTBAR, {})
end

function onReturn(scene, results, player)
    player:finish_event()
end
