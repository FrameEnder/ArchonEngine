-- Shadow Of The Destroyer (Lv. 82): 120 round the Monk (surely critical in Opo-opo form); Raptor form.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 1)
    end
    mnkUsePerfectBalance(effects, player)
    if mnkInOpoOpo(player) then
        effects:guarantee_critical()
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 120))
    mnkShift(effects, player, STATUS_RAPTOR_FORM)

    return effects
end
