-- Brutal Strength's (316) enemy (BNpcBase 28, layout 3633691): instant, 2.2 s after its first hit and 10 to 16 s
-- apart after, three times: status 17 on its target for 15 s and no damage (Bob's Chronofoil capture 0cafbf61 of
-- 2026-11-17: gone 15.2 s after the first; the second and third were off after 7.7 and 3.7 s, the last as it fell).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(17, 0, 15.0)

    return effects
end
