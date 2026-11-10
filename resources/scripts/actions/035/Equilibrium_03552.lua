-- Equilibrium (Lv. 58): heals the Warrior 1,200, and 200 over 15 s from 84.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_self(healAmount(player, 1200))
    if player.level >= 84 then
        effects:gain_effect_self(2681, 0, 15.0)
    end

    return effects
end
