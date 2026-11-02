-- An action of Haukke Manor's the first boss (BNpcBase 272) (territory 1040, with Duty Support or the Duty Finder): a
-- hit on its target and one more (328 to players, the timeline's recorded amount), every 20.2 s. From Bob's Chronofoil
-- captures 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast
-- are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
