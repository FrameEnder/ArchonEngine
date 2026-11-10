-- Gust Slash (Lv. 4), after Spinning Edge.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 240
local COMBO_POTENCY = 260

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        ninGainNinki(effects, player, 5)
    end
    ninBunshin(effects, player, false)

    return effects
end
