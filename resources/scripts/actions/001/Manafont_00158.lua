-- Manafont (Lv. 30): fully restores MP.
-- It leaves the element as it is (Astral Fire and Umbral Ice: Global.lua's blm* functions); Thunderhead as before.
local MP = 10000
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(MP)
    -- Granted when it isn't already up (retail doesn't refresh it.)
    if player:get_effect(EFFECT_THUNDERHEAD) == nil then
        effects:gain_effect_self(EFFECT_THUNDERHEAD, 0, THUNDERHEAD_DURATION)
    end

    return effects
end
