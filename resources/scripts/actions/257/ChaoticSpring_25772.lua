-- Chaotic Spring (Lv. 86): 140, 300 after Spiral Blow, and 45 over 24 s (rear bonuses aren't done: no positionals).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 140
    if in_combo then
        potency = 300
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    effects:damage_over_time(DRG_EFFECT_CHAOTIC_SPRING, 24.0, drgDamage(player, 45))

    return effects
end
