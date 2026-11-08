-- An action of the old marauder's kit that quest battle NPCs still use (Fracture: History Repeating's marauders,
-- quest battle 86, BNpcBase 1649): a slashing hit (11-15, the timeline's recorded amount) and its damage over time
-- (status 244), lost 18.7-18.8 s after it landed on the player and ticking 1-2 every 3 s (Bob's Chronofoil captures
-- of 2026-11-08). Retail's numbers, as fixed amounts.
local POTENCY = 100
local STATUS = 244
local DURATION = 18.0
local TICK = 2

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    effects:damage_over_time(STATUS, DURATION, TICK)

    return effects
end
