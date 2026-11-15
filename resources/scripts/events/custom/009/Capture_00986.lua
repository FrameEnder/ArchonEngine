-- CustomTalk 986: Keybound Brawler's registrar in Keybound Square (layout 12479940)
--
-- Written from retail alone (Connor's Chronofoil captures of 2026-11-03 to 11-09, 282 talks); the client's script
-- hasn't been dumped, so the file goes by the talk's number. Scene 0 (flags 0x2000) returns `[1, 0x2C0078]` to play:
-- the ContentEntry it names (120) is started, nested, and its scene 0 puts the duty in the Duty Finder
-- (`ContentEntryEventHandler`); both events finish as that returns. Closing the window cancels the scene (error 50,
-- nothing returned) and the event finishes.

SCENE_REGISTER = 0
SCENE_FLAGS = 0x2000
PLAY = 1

function onTalk(target, player)
    player:play_scene(SCENE_REGISTER, SCENE_FLAGS, {})
end

function onReturn(scene, results, player)
    if results[1] == PLAY and results[2] ~= nil and results[2] ~= 0 then
        -- The ContentEntry finishes this event with its own (`finish_nested_event`'s way.)
        player:start_event(results[2], EVENT_TYPE_NEST, 0)
        player:play_scene(0, SCENE_FLAGS, {})
        return
    end
    player:finish_event()
end
