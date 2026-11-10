-- Geirskogul (Lv. 60): 280 to the first enemy in a line, 50 % less to the rest; Life of the Dragon (15 % more damage, 20 s: the gauge's "lotd") from 70 and Nastrond Ready (20 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 280))
    if player.level >= 70 then
        effects:gauge_set("lotd", 200)
        effects:gain_effect_self(DRG_EFFECT_NASTROND_READY, 0, 20.0)
    end

    return effects
end
