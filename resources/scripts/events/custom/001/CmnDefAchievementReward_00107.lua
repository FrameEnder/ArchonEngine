-- Jonathas (Old Gridania), who hands out Achievement Certificates and exchanges them in his SpecialShop.
--
-- From the client's script (CmnDefAchievementReward_00107.luab) and Bob's Chronofoil capture b23b91ef (2026-10-26):
-- * scene 2 hands out the certificates owed; once it returns the server adds them (ActorControlSelf 378 for the
--   certificates and their total, log message 750) and plays the menu;
-- * scene 1 is the menu. With its parameter 0 it asks: the first choice returns MAIN_MENU_RESULT_SPECIALSHOP (2) and
--   the SpecialShop's event ID, which the server opens nested (retail: EventStart type 7, scene 0, flags 0x2001); the
--   second explains certificates and returns MAIN_MENU_RESULT_MENU (1), after which retail finished; the third
--   returns 0. Anything but 0 skips the question and returns 0. When the shop closes the menu plays again.
-- * scene 0 is a greeting only. Retail's first visit owed a certificate, so when scene 0 plays isn't recorded: here
--   it's for someone who has never been given one.

SCENE_GREETING = 00000
SCENE_MENU = 00001
SCENE_CERTIFICATES = 00002

MAIN_MENU_RESULT_SPECIALSHOP = 2

-- Every certificate ever given (the client script's CURRENCY_ACHIEVEMENT_TOTAL.)
CURRENCY_ACHIEVEMENT_TOTAL = 21173

-- Kawari's scene for returning from a nested event (not a real scene); the menu plays again then, rather than the
-- event finishing with the shop.
SCENE_RETURNED_FROM_NEST = 255
CONTINUES_AFTER_NESTED = true

function playMenu(player)
    player:play_scene(SCENE_MENU, HIDE_HOTBAR, {0})
end

function onTalk(target, player)
    if player:achievement_certificates_owed() > 0 then
        player:play_scene(SCENE_CERTIFICATES, HIDE_HOTBAR, {})
    elseif player:player_currency(CURRENCY_ACHIEVEMENT_TOTAL) > 0 then
        playMenu(player)
    else
        player:play_scene(SCENE_GREETING, HIDE_HOTBAR, {})
    end
end

function onReturn(scene, results, player)
    if scene == SCENE_CERTIFICATES then
        player:give_achievement_certificates()
        playMenu(player)
    elseif scene == SCENE_MENU and results[1] == MAIN_MENU_RESULT_SPECIALSHOP and results[2] ~= nil then
        player:start_event(results[2], EVENT_TYPE_NEST, 0)
        player:play_scene(0, HIDE_HOTBAR | NO_DEFAULT_CAMERA, {})
    elseif scene == SCENE_RETURNED_FROM_NEST then
        playMenu(player)
    else
        player:finish_event()
    end
end
