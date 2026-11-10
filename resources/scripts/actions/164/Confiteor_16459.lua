-- Confiteor (Lv. 80): 500 on the target, 60 % less round it, 1,000 under Requiescat; only with Confiteor Ready; heals 400.
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
    effects:consume_effect(PLD_EFFECT_CONFITEOR_READY)
    pldSpellHeal(effects, player)

    return effects
end
