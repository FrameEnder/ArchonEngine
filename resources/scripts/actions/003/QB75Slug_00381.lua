-- Aqueous Discharge (381): the slug in Seer Folly's quest battle 75 (BNpcBase 1677), a 2.5 s cast 3 yalms round itself
-- every 7.2 s; 250-256 to Raya-O-Senna and A-Ruhn-Senna, half its casts missed (Bob's Chronofoil capture 8d550e33,
-- 2026-11-05.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
