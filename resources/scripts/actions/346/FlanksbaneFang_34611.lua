-- Flanksbane Fang (Lv. 30): 340 (the positional's 60 isn't done), 100 more with its venom; Hindsbane Venom (60 s); 10 Serpent Offerings from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 340 + vprBonus(effects, player, VPR_EFFECT_FLANKSBANE_VENOM, 100))
    vprVenom(effects, player, VPR_EFFECT_HINDSBANE_VENOM)
    vprOfferings(effects, player, 10)

    return effects
end
