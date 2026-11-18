-- Cape Westwind's helper's 28867: no cast, from the boss's spot on whoever struck his raised shield from in front, a
-- second after the blow: 769 and the Knockback sheet's row 7 (Bob's Chronofoil capture 9ba7d981 of 2026-11-17, visit
-- 0.6: once in four runs, the only time anyone hit him from in front while statuses 680 and 2195 were on him.) The
-- helpers' timeline's `recorded_amounts` scale it (Helper_9020.json).
local POTENCY = 100
local KNOCKBACK = 7

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
