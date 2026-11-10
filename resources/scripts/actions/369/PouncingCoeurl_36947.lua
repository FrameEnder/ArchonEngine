-- Pouncing Coeurl (Lv. 92): 310, 460 with Coeurl's Fury (flank bonuses aren't done: no positionals); Opo-opo form.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 3)
    end
    mnkUsePerfectBalance(effects, player)
    local potency = 310
    if mnkSpendFury(effects, player, "coeurl_fury") then
        potency = 460
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_OPO_OPO_FORM)

    return effects
end
