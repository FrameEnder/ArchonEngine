-- The Ultimate Weapon's last battle's helper's 29788: no cast, on the player from the middle, 0.9 s after the gauge
-- fills: the Knockback sheet's row 68 and status 1614, which lasted until the player fell (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) What the status does isn't known.
local KNOCKBACK = 68
local STATUS = 1614
local DURATION = 600.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:knockback(KNOCKBACK)
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
