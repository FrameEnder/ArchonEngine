-- Gust Slash (Lv. 4), after Spinning Edge.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 240
local COMBO_POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
