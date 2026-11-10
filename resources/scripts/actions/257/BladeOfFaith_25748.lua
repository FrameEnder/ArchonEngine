-- Blade Of Faith (Lv. 90): 260 on the target, 60 % less round it, 760 under Requiescat; after Confiteor; heals 400.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 260
    if player:get_effect(PLD_EFFECT_REQUIESCAT) ~= nil then
        potency = 760
        effects:consume_effect_stack(PLD_EFFECT_REQUIESCAT)
    end
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    pldSpellHeal(effects, player)

    return effects
end
