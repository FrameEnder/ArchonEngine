-- An action of Haukke Manor's the pair's fighter (BNpcBase 274) (territory 1040, with Duty Support or the Duty Finder):
-- a drain every 26.7 s: 392 to players and 387 back to itself. From Bob's Chronofoil captures 5d9c7d7f and e7580e0d
-- (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:heal_self(387)

    return effects
end
