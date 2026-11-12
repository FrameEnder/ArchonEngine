-- Shining Topaz, the Topaz Carbuncle's in Tactical Planning (quest battle 50, BNpcBase 2049): 5 yalms round itself (the
-- Action sheet's), every 17.4 s for 3-4 (Bob's Kawari Recorder run of 2026-11-12; its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
