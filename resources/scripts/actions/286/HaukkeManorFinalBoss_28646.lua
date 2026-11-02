-- An action of Haukke Manor's the last boss (BNpcBase 14504) (territory 1040, with Duty Support or the Duty Finder): on
-- a player every 63.1 s: 260 and status 66 for 6 s (lost 6.0 s later all three times). From Bob's Chronofoil captures
-- 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast are the
-- Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(66, 0, 6.0)

    return effects
end
