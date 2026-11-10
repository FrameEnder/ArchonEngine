-- Piercing Talon (Lv. 15): ranged; stronger with Enhanced Piercing Talon (from Elusive Jump), which it uses up.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 150
local ENHANCED_POTENCY = 230
local EFFECT_ENHANCED_PIERCING_TALON = 1870

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = POTENCY
    if player:get_effect(EFFECT_ENHANCED_PIERCING_TALON) ~= nil then
        potency = ENHANCED_POTENCY
        effects:consume_effect(EFFECT_ENHANCED_PIERCING_TALON)
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))

    return effects
end
