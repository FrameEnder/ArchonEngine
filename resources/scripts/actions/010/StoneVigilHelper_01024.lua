-- Sheet of Ice: the last boss's helpers' (BNpcBase 9020) circle on each of the party (2.7 s) as it flies, as the
-- director says (1939 to the player). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty
-- Support, three wipes). Its area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the hit to
-- retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
