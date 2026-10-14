-- Generic aetheryte, use this for all of the aethernet shards

-- Scenes
SCENE_00000                 = 00000 -- does nothing
SCENE_00001                 = 00001 -- does nothing
SCENE_SHOW_MENU             = 00002 -- aetheryte menu
SCENE_HAVE_AETHERNET_ACCESS = 00003 -- "you have aethernet access" message and vfx
SCENE_00100                 = 00000 -- "According to the message engraved in the base, special permission is required to use this aetheryte." (Eulmore-specific)
SCENE_00200                 = 00200 -- "The aetheryte has ceased functioning." (Eulmore-specific)

function aetheryteId()
    return EVENT_ID & 0xFFFF
end

-- EventAction "Attuning": the attunement animation with a 5 second progress bar (Sapphire's ACTION_ATTUNE.)
EVENT_ACTION_ATTUNE = 19

function onTalk(target, player)
    if not player:has_aetheryte(aetheryteId()) then
        -- Attune first, the rest happens once the progress bar finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_ATTUNE)
        return
    end

    player:play_scene(SCENE_SHOW_MENU, HIDE_HOTBAR, {0})
end

function onEventActionCast(player)
    player:unlock_aetheryte(1, aetheryteId())
    -- "You have aethernet access" and its effect, then the event ends.
    player:play_scene(SCENE_HAVE_AETHERNET_ACCESS, HIDE_HOTBAR, {0})
end

function onReturn(scene, results, player)
    local AETHERNET_MENU_CANCEL = 0
    local destination = results[1]

    if scene == SCENE_SHOW_MENU then
        if destination ~= AETHERNET_MENU_CANCEL then
            player:finish_event() -- Need to finish the event here, because warping does not return to this callback (the game will crash or softlock otherwise)
            player:warp_aetheryte(destination)
            return
        end
    --elseif scene == HAVE_AETHERNET_ACCESS then
        -- TODO: attunement logic
    end

    player:finish_event()
end
