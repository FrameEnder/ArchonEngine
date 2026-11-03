-- An action of Brayflox's Longstop's 1042 (BNpcBase 1042) (territory 1041): no damage, status 7 for 5 s on its
-- target (once). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its
-- area and cast are the Action sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(7, 0, 5.0)

    return effects
end
