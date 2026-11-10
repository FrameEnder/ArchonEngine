-- Lance Barrage (Lv. 96): 130, 340 after True Thrust or Raiden Thrust.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 130
    if in_combo then
        potency = 340
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))

    return effects
end
