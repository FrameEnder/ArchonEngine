-- Hakke Mujinsatsu (Lv. 52): 100 round the Ninja, 120 after Death Blossom (5 Ninki).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 100
    if in_combo then
        potency = 120
        ninGainNinki(effects, player, 5)
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
