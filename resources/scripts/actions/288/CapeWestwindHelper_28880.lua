-- Cape Westwind's helper's 28880: a 0.2 s cast on the player from where they stand, 0.9 s after the boss's 28879:
-- the Knockback sheet's row 17, which took them 6.7 yalms towards him (Bob's Chronofoil capture 667a68c5 of 2026-11-16, one run.)
local KNOCKBACK = 17

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:knockback(KNOCKBACK)

    return effects
end
