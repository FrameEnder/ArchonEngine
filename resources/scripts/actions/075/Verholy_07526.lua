-- Verholy (Lv. 70): 650 to the first enemy, 55 % less round it; 11 White Mana; three Mana Stacks; Verstone Ready 20 % of the time (always with less White than Black).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmFinisher(effects, player, 11, 0, RDM_EFFECT_VERSTONE_READY, player:gauge("black_mana") > player:gauge("white_mana"))

    return effects
end
