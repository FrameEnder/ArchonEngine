-- Fang And Claw (Lv. 56): 140, 300 after Heavens' Thrust (flank bonuses aren't done: no positionals).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 140
    if in_combo then
        potency = 300
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))

    return effects
end
