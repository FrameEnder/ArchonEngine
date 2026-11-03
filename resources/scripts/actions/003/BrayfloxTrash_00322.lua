-- An action of Brayflox's Longstop's 1041s (BNpcBase 1041) (territory 1041): a hit on its target from range, every
-- 3.9 s between its auto-attacks (138 to players in the Duty Finder run, 48-50 to the Duty Support tank). From Bob's
-- Chronofoil captures 44df02fc (visit 0.20, Duty Support) and c9145fdc (visit 0.14, the Duty Finder), 2026-11-03. Its
-- area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
