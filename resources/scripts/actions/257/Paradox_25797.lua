-- Paradox (Lv. 90): 540; 1,600 MP, none in Umbral Ice; in Astral Fire Firestarter; only with Paradox ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(classPotency(player, 540, TRAITS_THM_MAIM_AND_MEND) * blmEnochian(player))))
    effects:gauge_set("paradox", 0)
    if blmElement(player) < 0 then
        effects:set_mp_cost(0)
    else
        effects:set_mp_cost(1600)
        effects:gain_effect_self(165, 0, 0.0)
    end

    return effects
end
