-- Prominence (Lv. 40), after Total Eclipse: all nearby enemies. Combo bonus: restores MP and, from Enhanced
-- Prominence (72), Divine Might (30 s).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local COMBO_POTENCY = 220
local COMBO_MP = 1000

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        effects:restore_mp(COMBO_MP)
        if player.level >= 72 then
            effects:gain_effect_self(PLD_EFFECT_DIVINE_MIGHT, 0, 30.0)
        end
    end

    return effects
end
