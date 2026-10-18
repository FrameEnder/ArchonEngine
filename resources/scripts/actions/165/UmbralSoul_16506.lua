-- Umbral Soul (Lv. 35): grants Umbral Ice and an Umbral Heart.
-- NOTE: Astral Fire and Umbral Ice (the job gauge) aren't implemented: no potency or MP changes from them. Since 7.x Thunder spells need Thunderhead; without the gauge, changing element (this) grants it.
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_THUNDERHEAD, 0, THUNDERHEAD_DURATION)

    return effects
end
