-- Living Shadow (Lv. 80): the simulacrum, 22 s: six attacks with the next weaponskills (`drkShadowHit`); Scorn, 30 s, from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_set("shadow_hits", 6)
    if player.level >= 100 then
        effects:gain_effect_self(DRK_EFFECT_SCORN, 0, 30.0)
    end

    return effects
end
