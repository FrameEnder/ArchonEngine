-- Titan's Geocrush (the Navel, BNpcBase 246): a 4.2 s cast from the arena's middle, where he leapt, on everyone (the
-- Action sheet's area), weaker further out: 971-1017 to players 4.6-4.8 yalms from him, 347-480 at 18.8-22.1
-- (Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel). His timeline's `recorded_amounts` scale it, by how far it lands (`by_distance`).
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
