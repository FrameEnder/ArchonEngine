-- Disembowel (Lv. 18), after True Thrust. Combo bonus: Power Surge (+10% damage, 30 s).
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 140
local COMBO_POTENCY = 200
local EFFECT_POWER_SURGE = 2720
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    if in_combo then
        effects:gain_effect_self(EFFECT_POWER_SURGE, 0, DURATION)
    end

    return effects
end
