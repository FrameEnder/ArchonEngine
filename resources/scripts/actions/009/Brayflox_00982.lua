-- An action of Brayflox's Longstop's 1048s and its third boss (BNpcBases 1048, 427) (territory 1041): a 3.5 s cast
-- round itself, no damage: status 60 on those it reaches (once on a player in the Duty Finder run; how long it lasts
-- wasn't seen, 15 s is a guess). From Bob's Chronofoil captures 44df02fc (visit 0.20, Duty Support) and c9145fdc
-- (visit 0.14, the Duty Finder), 2026-11-03. Its area and cast are the Action sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(60, 0, 15.0)

    return effects
end
