-- Storm's Path (Lv. 26), after Maim. Combo bonus: restores own HP.
-- Combo bonus: Beast Gauge +20 (Global.lua's warGainBeast).
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 220
local COMBO_POTENCY = 380
local HEAL_POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        warBerserk(effects, player)
    end
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        effects:heal_self(healAmount(player, HEAL_POTENCY))
        warGainBeast(effects, player, 20)
    end

    return effects
end
