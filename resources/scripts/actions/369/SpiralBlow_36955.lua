-- Spiral Blow (Lv. 96): 140, 300 after True Thrust or Raiden Thrust; the combo gives Power Surge (30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 140
    if in_combo then
        potency = 300
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    if in_combo then
        effects:gain_effect_self(DRG_EFFECT_POWER_SURGE, 0, 30.0)
    end

    return effects
end
