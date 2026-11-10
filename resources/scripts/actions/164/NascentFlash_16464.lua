-- Nascent Flash (Lv. 76): the Warrior heals 400 with each weaponskill, the target as much and takes 10 % less (8 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(1857, 0, 8.0)
    effects:gain_effect(1858, 0, 8.0)

    return effects
end
