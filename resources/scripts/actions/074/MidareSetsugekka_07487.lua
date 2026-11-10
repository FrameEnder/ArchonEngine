-- Midare Setsugekka (Lv. 50): a sure critical 680; three Sen; a Meditation stack; Tsubame-gaeshi from 74.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:guarantee_critical()
    samDamage(effects, player, 680)
    samIaijutsu(effects, player, 3)

    return effects
end
