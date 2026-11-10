-- Dark Missionary (Lv. 66): the party near takes 5 % less physical and 10 % less magic damage, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DRK_EFFECT_DARK_MISSIONARY, 0, 15.0)

    return effects
end
