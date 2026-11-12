-- Rock Buster, Titan-Egi's in Austerities of Earth (quest battle 80, BNpcBase 2203): a hit on its target, 52-79 on the
-- player and 75 on Y'mhitra (Bob's Kawari Recorder run of 2026-11-12; its timeline's recorded amounts.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
