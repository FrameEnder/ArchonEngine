-- Requiescat (Lv. 68): 320; four stacks of Requiescat and Confiteor Ready (from 80), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, 320))
    effects:gain_effect_self(PLD_EFFECT_REQUIESCAT, 4, 30.0)
    if player.level >= 80 then
        effects:gain_effect_self(PLD_EFFECT_CONFITEOR_READY, 0, 30.0)
    end

    return effects
end
