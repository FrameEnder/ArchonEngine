-- Manafont (Lv. 30): fully restores MP.
-- NOTE: Astral Fire and Umbral Ice (the job gauge) aren't implemented: no potency or MP changes from them. Since 7.x Thunder spells need Thunderhead; without the gauge, changing element (this) grants it.
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
