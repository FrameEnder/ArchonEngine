-- The Jade Stoa's Hakutei (10827): a 19.7 s cast on everyone: 783 for 8 s. From Bob's Chronofoil capture 84700114
-- (2026-11-24, the Jade Stoa unsynced). Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 100
local STATUS = 783
local DURATION = 8.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, 0, DURATION)
    return effects
end
