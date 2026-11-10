-- Stardiver (Lv. 80): 840 to the first enemy round it, 40 % less to the rest; only in Life of the Dragon; Starcross Ready (20 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 840))
    if player.level >= 100 then
        effects:gain_effect_self(DRG_EFFECT_STARCROSS_READY, 0, 20.0)
    end

    return effects
end
