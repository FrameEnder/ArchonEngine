-- Gouge, the Topaz Carbuncle's in Tactical Planning (quest battle 50, BNpcBase 2049): a hit on its target every 3.4-4.8
-- s for 1-2 (Bob's Kawari Recorder run of 2026-11-12; its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
