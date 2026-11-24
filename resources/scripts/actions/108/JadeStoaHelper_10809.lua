-- The Jade Stoa's helper (10809): no cast: each step outward of an eight-circle line, 1.07 s apart: 482 (75) for 8 s.
-- From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is the Action sheet's; the
-- timeline's `recorded_amounts` scale it.

local POTENCY = 100
local STATUS = 482
local PARAM = 75
local DURATION = 8.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)
    return effects
end
