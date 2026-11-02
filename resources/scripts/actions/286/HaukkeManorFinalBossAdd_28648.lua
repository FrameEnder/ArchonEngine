-- An action of Haukke Manor's the last boss's 14505 (territory 1040, with Duty Support or the Duty Finder): a gaze
-- every 34.2 s on up to four: 42 of 48 missed (looked away); the rest took 247 and status 1511 for 7-9 s. From Bob's
-- Chronofoil captures 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its
-- area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gaze()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(1511, 0, 8.0)

    return effects
end
