-- The Porta Decumana's second boss's 29015: a 3.7 s cast on itself from the middle 53.4 s in: everyone thrown to the
-- arena's edge (the Knockback sheet's row 13; effect type 31 with it), no damage; the four of base 14594 come 0.8 s
-- later (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.)

local KNOCKBACK = 13

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:knockback(KNOCKBACK)

    return effects
end
