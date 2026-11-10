-- Barrage (Lv. 38): the next Wide Volley is stronger, 10 s.
local EFFECT_BARRAGE = 128
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_BARRAGE, 0, DURATION)
    -- Resonant Arrow Ready (30 s) from Enhanced Barrage (96).
    if player.level >= 96 then
        effects:gain_effect_self(BRD_EFFECT_RESONANT_ARROW_READY, 0, 30.0)
    end

    return effects
end
