-- Burning Strike, Ifrit-Egi's in Austerities of Flame (quest battle 79, BNpcBase 1513): a hit on its target, 39 on the
-- player each of three times (Bob's Kawari Recorder run of 2026-11-12; its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
