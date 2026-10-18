-- Prominence (Lv. 40), after Total Eclipse: all nearby enemies. Combo bonus: restores MP.
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
    end

    return effects
end
