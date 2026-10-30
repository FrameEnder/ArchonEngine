-- The Thousand Maws of Toto-Rak's last boss (BNpcBase 259): a 2.2 s cast round itself every 36.6 s (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 156-248 to
-- those still in it, most stood out of it.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
