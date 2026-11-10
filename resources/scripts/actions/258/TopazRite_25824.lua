-- Topaz Rite (Lv. 72): 340; a stack of earth attunement; Titan's Favor from 86.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnGem(effects, player, 2, 340)
    if player.level >= 86 then
        effects:gain_effect_self(SMN_EFFECT_TITANS_FAVOR, 0, 30.0)
    end

    return effects
end
