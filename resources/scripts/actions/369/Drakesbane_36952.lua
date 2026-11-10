-- Drakesbane (Lv. 64): 460 after Wheeling Thrust or Fang and Claw; Draconian Fire (30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 460
    if in_combo then
        potency = 460
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    effects:gain_effect_self(DRG_EFFECT_DRACONIAN_FIRE, 0, 30.0)

    return effects
end
