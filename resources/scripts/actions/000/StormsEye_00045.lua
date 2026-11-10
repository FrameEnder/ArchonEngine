-- Storm's Eye (Lv. 50), after Maim. Combo bonus: Surging Tempest (+10% damage, 30 s).
-- Combo bonus: Beast Gauge +10 (Global.lua's warGainBeast).
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 220
local COMBO_POTENCY = 380
local EFFECT_SURGING_TEMPEST = 2677
local DURATION = 30.0
local MAX_DURATION = 60.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        -- A refresh adds to the time left, up to 60 s (retail, Bob's Chronofoil captures.)
        local left = player:effect_time_left(EFFECT_SURGING_TEMPEST) or 0
        effects:gain_effect_self(EFFECT_SURGING_TEMPEST, 0, math.min(left + DURATION, MAX_DURATION))
        warGainBeast(effects, player, 10)
    end

    return effects
end
