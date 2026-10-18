-- Dragonfire Dive (Lv. 50): the target and enemies near it, 50% less after the first.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 500
local FALLOFF = 50

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(FALLOFF)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))

    return effects
end
