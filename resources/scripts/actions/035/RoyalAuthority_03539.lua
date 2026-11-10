-- Royal Authority (Lv. 60): 200, 460 after Riot Blade; the combo gives Divine Might (from 64) and Atonement Ready (from 76), 30 s each.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 200
    if in_combo then
        potency = 460
        if player.level >= 64 then
            effects:gain_effect_self(PLD_EFFECT_DIVINE_MIGHT, 0, 30.0)
        end
        if player.level >= 76 then
            effects:gain_effect_self(PLD_EFFECT_ATONEMENT_READY, 0, 30.0)
        end
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
