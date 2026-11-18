-- The Ultimate Weapon's last battle's helper's 29788: no cast, on the player from the middle, 0.9 s after the gauge
-- fills: the Attract sheet's row 68 (effect kind 32, as Cape Westwind's 28880: the player drawn to the middle; it was
-- taken for a Knockback row until 2026-11-18) and status 1614 with no timer, param 6174 (Kawari Witness), which
-- lasted until the player fell (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17
-- and 6320ae56 of 2026-11-18, three runs.) What the status does isn't known.
local ATTRACT = 68
local STATUS = 1614
local STATUS_PARAM = 6174
local DURATION = 0.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:attract(ATTRACT)
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION)
    return effects
end
