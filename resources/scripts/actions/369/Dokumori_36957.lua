-- Dokumori (Lv. 66): 400 to the enemies in a cone, 5 % more damage taken for 20 s; 40 Ninki; Higi (30 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 400))
    effects:gain_effect(NIN_EFFECT_DOKUMORI, 0, 20.0)
    ninGainNinki(effects, player, 40)
    if player.level >= 96 then
        effects:gain_effect_self(NIN_EFFECT_HIGI, 0, 30.0)
    end

    return effects
end
