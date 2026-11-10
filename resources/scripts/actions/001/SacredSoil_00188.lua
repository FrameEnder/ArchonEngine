-- Sacred Soil (Lv. 50): an area where the party takes 10 % less damage, 15 s (the caster gets the status: no ground areas yet), with a Regen of 100 from 78; an Aetherflow.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(298, 0, 15.0)
    schSpendAetherflow(effects, player, false)

    return effects
end
