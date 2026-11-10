-- Full Thrust (Lv. 26), after Vorpal Thrust.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local COMBO_POTENCY = 380

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))

    return effects
end
