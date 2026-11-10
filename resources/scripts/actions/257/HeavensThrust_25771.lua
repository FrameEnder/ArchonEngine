-- Heavens Thrust (Lv. 86): 160, 460 after Lance Barrage.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 160
    if in_combo then
        potency = 460
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))

    return effects
end
