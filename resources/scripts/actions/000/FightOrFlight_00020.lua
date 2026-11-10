-- Fight or Flight (Lv. 2): damage dealt +25% for 20 s; Goring Blade Ready (30 s) from Enhanced Fight or Flight (54).
local EFFECT_FIGHT_OR_FLIGHT = 76
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_FIGHT_OR_FLIGHT, 0, DURATION)
    if player.level >= 54 then
        effects:gain_effect_self(PLD_EFFECT_GORING_BLADE_READY, 0, 30.0)
    end

    return effects
end
