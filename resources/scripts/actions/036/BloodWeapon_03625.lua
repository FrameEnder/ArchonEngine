-- Blood Weapon (Lv. 35): three stacks of Blood Weapon, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DRK_EFFECT_BLOOD_WEAPON, 3, 15.0)

    return effects
end
