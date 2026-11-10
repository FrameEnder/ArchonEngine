-- Star Prism (Lv. 100): 1,100 to the target, 70 % less round it, and 400 healed for the party (here: the Pictomancer); Starstruck spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 1100, 70)
    effects:heal_self(healAmount(player, 400))
    effects:consume_effect(PCT_EFFECT_STARSTRUCK)

    return effects
end
