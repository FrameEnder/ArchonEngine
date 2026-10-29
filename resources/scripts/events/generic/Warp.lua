-- Generic handler for Warps (lifts, doors, ferries, airship ticketers): a yes/no, then the warp.
--
-- As retail played them (Bob's Chronofoil captures cc62ad94 and deaf7e28; the Warp and WarpCondition sheets, read
-- through `player:warp_info`):
-- * When the player meets the warp's WarpCondition (its quests, class level and fare) and it has a
--   ConditionSuccessEvent, that DefaultTalk plays first, nested (EventStart type 7, its scene 0, flags 0x2000), then
--   the question (Aleport's ferry, warp 37; the lift to Chocobo Square, warp 105).
-- * When the player doesn't meet it, the ConditionFailEvent plays instead of the question, and nothing else happens:
--   the client itself refuses the question then (error code 50, Bob's bundle 20261029-154412: the Ul'dah ticketers,
--   warps 29 and 30, with too little gil; the Gold Saucer's, warp 108, before It Could Happen to You). Retail's fail
--   talk wasn't captured; it's played the way the success talk is.
-- * Yes (results[1] == 1) on a warp with a StartCutscene (the airships) plays scene 1000 (retail's flags 0x482402, warp
--   31: the cutscene of the flight), then warps. The fare (WarpCondition's Gil) is taken as the warp goes (retail's
--   gil slot update came with PrepareZoning: Aleport's 40 gil, Limsa Lominsa to Ul'dah's 120.)

CONTINUES_AFTER_NESTED = true

SCENE_QUESTION = 0
SCENE_TRIP = 1000
-- Kawari's scene for returning from a nested event (not a real scene.)
SCENE_RETURNED_FROM_NEST = 255

TRIP_FLAGS = 0x482402

warp = nil
refused = false

function talk(player, default_talk)
    player:start_event(default_talk, EVENT_TYPE_NEST, 0)
    player:play_scene(0, HIDE_HOTBAR, {})
end

function onTalk(target, player)
    warp = player:warp_info(EVENT_ID)
    refused = false
    if not warp.met and warp.fail_event ~= 0 then
        refused = true
        talk(player, warp.fail_event)
        return
    end
    if warp.met and warp.success_event ~= 0 then
        talk(player, warp.success_event)
        return
    end
    player:play_scene(SCENE_QUESTION, HIDE_HOTBAR, {0})
end

function go(player)
    if warp ~= nil and warp.gil > 0 then
        player:modify_currency(CURRENCY_GIL, -warp.gil)
    end
    player:finish_event()
    player:warp(EVENT_ID)
end

function onReturn(scene, results, player)
    if scene == SCENE_RETURNED_FROM_NEST then
        if refused then
            player:finish_event()
        else
            player:play_scene(SCENE_QUESTION, HIDE_HOTBAR, {0})
        end
        return
    end

    if scene == SCENE_QUESTION and results[1] == 1 then
        if warp ~= nil and warp.cutscene ~= 0 then
            player:play_scene(SCENE_TRIP, TRIP_FLAGS, {})
            return
        end
        go(player)
        return
    end

    if scene == SCENE_TRIP then
        go(player)
        return
    end

    player:finish_event()
end
