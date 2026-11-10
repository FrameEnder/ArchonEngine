-- Verflare (Lv. 68): 650 to the first enemy, 55 % less round it; 11 Black Mana; three Mana Stacks; Verfire Ready 20 % of the time (always with less Black than White).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmFinisher(effects, player, 0, 11, RDM_EFFECT_VERFIRE_READY, player:gauge("white_mana") > player:gauge("black_mana"))

    return effects
end
