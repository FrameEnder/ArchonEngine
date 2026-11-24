-- The Jade Stoa's orb (10824): no cast: goes off on whoever comes within 1.5 yalms: Vulnerability Up (202) for 90 s.
-- From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is the Action sheet's; the
-- timeline's `recorded_amounts` scale it.

local POTENCY = 100
local VULNERABILITY_UP = 202
local PARAM = 10
local DURATION = 90.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(VULNERABILITY_UP, PARAM, DURATION)
    return effects
end
