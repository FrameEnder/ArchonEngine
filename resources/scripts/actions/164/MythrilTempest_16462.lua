-- Mythril Tempest (Lv. 40), after Overpower: all nearby enemies. Combo bonus: Surging Tempest (+10% damage, 30 s).
-- Combo bonus: Beast Gauge +20 from Mastering the Beast (74; Global.lua's warGainBeast).
local MASTERING_THE_BEAST_LEVEL = 74
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local COMBO_POTENCY = 140
local EFFECT_SURGING_TEMPEST = 2677
local DURATION = 30.0

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
        effects:gain_effect_self(EFFECT_SURGING_TEMPEST, 0, DURATION)
        if player.level >= MASTERING_THE_BEAST_LEVEL then
            warGainBeast(effects, player, 20)
        end
    end

    return effects
end
