-- Riot Blade (Lv. 4), after Fast Blade. Combo bonus: restores MP.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 170
local COMBO_POTENCY = 330
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
