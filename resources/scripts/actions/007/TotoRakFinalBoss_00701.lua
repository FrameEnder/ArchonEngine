-- The Thousand Maws of Toto-Rak's last boss (BNpcBase 259): a 2.2 s cast round itself every 36.6 s (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 156-248 to
-- those still in it, most stood out of it.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100
-- Silkscreen knocks whoever it hits back: the Knockback sheet's row 1 after the damage in every hit, the Toto-Rak boss's
-- three and Seer Folly's quest battle 75 boss's seven (BNpcBase 1674, which uses it too: Bob's Chronofoil capture
-- 8d550e33, 2026-11-05, its targets 9-10 yalms further from it after; `1F 00 00 00 00 00 01 00`.)
local KNOCKBACK = 1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
