-- Twin Snakes (Lv. 18): changes to coeurl form.
-- Gives Raptor's Fury (the gauge's "raptor_fury").
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 340
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 2)
    end
    mnkUsePerfectBalance(effects, player)
    if isPlayerCaster(player) then
        effects:gauge_set("raptor_fury", 1)
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    mnkShift(effects, player, STATUS_COEURL_FORM)

    return effects
end
