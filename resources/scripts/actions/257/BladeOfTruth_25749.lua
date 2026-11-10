-- Blade Of Truth (Lv. 90): 380 on the target, 60 % less round it, 880 under Requiescat; after Blade of Faith; heals 400.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 380
    if player:get_effect(PLD_EFFECT_REQUIESCAT) ~= nil then
        potency = 880
        effects:consume_effect_stack(PLD_EFFECT_REQUIESCAT)
    end
    effects:aoe_falloff(60)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    pldSpellHeal(effects, player)

    return effects
end
