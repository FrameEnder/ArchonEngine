-- Maim (Lv. 4), after Heavy Swing.
-- NOTE: the job gauge isn't implemented, so the combo's Beast Gauge (+10) isn't given.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 190
local COMBO_POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
