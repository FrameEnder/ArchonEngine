-- Leaping Opo (Lv. 92): 260, 460 with Opo-opo's Fury; surely critical in Opo-opo form; Raptor form.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 1)
    end
    mnkUsePerfectBalance(effects, player)
    local potency = 260
    if mnkSpendFury(effects, player, "opo_fury") then
        potency = 460
    end
    if mnkInOpoOpo(player) then
        effects:guarantee_critical()
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_RAPTOR_FORM)

    return effects
end
