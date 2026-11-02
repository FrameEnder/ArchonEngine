-- An action of Haukke Manor's the manor's 1363s (BNpcBase 1363) (territory 1040, with Duty Support or the Duty Finder):
-- a hit every 6.5 s: 60 to players. From Bob's Chronofoil captures 5d9c7d7f and e7580e0d (2026-11-02): one run with
-- Duty Support and three Duty Finder runs. Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
