-- Blade Of Valor (Lv. 90): 500 on the target, 60 % less round it, 1000 under Requiescat; after Blade of Truth; gives Blade of Honor Ready (30 s) from 100; heals 400.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 500
    if player:get_effect(PLD_EFFECT_REQUIESCAT) ~= nil then
        potency = 1000
        effects:consume_effect_stack(PLD_EFFECT_REQUIESCAT)
    end
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    pldSpellHeal(effects, player)
    if player.level >= 100 then
        effects:gain_effect_self(PLD_EFFECT_BLADE_OF_HONOR_READY, 0, 30.0)
    end

    return effects
end
