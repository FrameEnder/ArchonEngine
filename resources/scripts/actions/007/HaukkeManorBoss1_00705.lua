-- An action of Haukke Manor's the first boss (BNpcBase 272) (territory 1040, with Duty Support or the Duty Finder): on
-- a player, every 50.8 s: 325 and status 66 for 11 s (lost 11.0 s later all four times). From Bob's Chronofoil captures
-- 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast are the
-- Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(66, 0, 11.0)

    return effects
end
