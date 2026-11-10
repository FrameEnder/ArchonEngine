-- Rising Raptor (Lv. 92): 340, 540 with Raptor's Fury; Coeurl form.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 2)
    end
    mnkUsePerfectBalance(effects, player)
    local potency = 340
    if mnkSpendFury(effects, player, "raptor_fury") then
        potency = 540
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_COEURL_FORM)

    return effects
end
