-- An action quest battle NPCs use (quest battle 64's BNpcBase 1874): 202 on the ally it hit and status 119 (Connor's
-- Chronofoil capture e834e436 of 2026-11-07: the ActionEffect), scaled to the timeline's recorded amount. The status's
-- length and ticks weren't seen. Not verified for players who use it.
local POTENCY = 150
local STATUS = 119
local DURATION = 18.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
