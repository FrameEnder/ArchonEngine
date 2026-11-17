-- The Ultimate Weapon's last battle's ally's 29344: no cast, on the fallen player from the middle, 36.3 s after they
-- fell: statuses 3123 (param 150) and 2056, which lasted to the end; the player's HP was twice what it had been from
-- then (not done here), and they got up 24.1 s later (the director's, content/questbattle/5061.lua) (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) What the statuses do isn't
-- known.
local STATUS_1 = 3123
local STATUS_1_PARAM = 150
local STATUS_2 = 2056
local DURATION = 3600.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS_1, STATUS_1_PARAM, DURATION)
    effects:gain_effect(STATUS_2, 0, DURATION)

    return effects
end
