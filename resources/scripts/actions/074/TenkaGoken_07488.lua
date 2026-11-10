-- Tenka Goken (Lv. 40): 300 to all enemies near; two Sen; a Meditation stack; Tsubame-gaeshi from 74.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 300)
    samIaijutsu(effects, player, 2)

    return effects
end
