-- Energy Siphon (Lv. 52): 100 to the target and round it; two Aetherflow; Further Ruin (60 s) from 62.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 100, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_aetherflow()
    if player.level >= 62 then
        effects:gain_effect_self(SMN_EFFECT_FURTHER_RUIN, 0, 60.0)
    end

    return effects
end
