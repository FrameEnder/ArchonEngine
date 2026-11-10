-- Bunshin (Lv. 80): five shadow stacks (30 s): each weaponskill a shadow's hit of 160 (80 for area ones); Phantom Kamaitachi Ready (45 s) from 82; 50 Ninki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(NIN_EFFECT_BUNSHIN, 5, 30.0)
    if player.level >= 82 then
        effects:gain_effect_self(NIN_EFFECT_PHANTOM_KAMAITACHI_READY, 0, 45.0)
    end
    effects:gauge_add("ninki", -50)

    return effects
end
