-- Circle of Scorn (Lv. 50): all nearby enemies, then damage over time.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 140
local DOT_POTENCY = 30
local DURATION = 15.0
local EFFECT_CIRCLE_OF_SCORN = 248

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    effects:damage_over_time(EFFECT_CIRCLE_OF_SCORN, DURATION, physicalDamage(player, DOT_POTENCY))

    return effects
end
