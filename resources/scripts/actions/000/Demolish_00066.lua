-- Demolish (Lv. 30): changes to opo-opo form.
-- Gives two Coeurl's Fury (the gauge's "coeurl_fury"). NOTE: the rear bonus (420) isn't implemented (no positionals).
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 280
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 3)
    end
    mnkUsePerfectBalance(effects, player)
    if isPlayerCaster(player) then
        effects:gauge_set("coeurl_fury", 2)
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    mnkShift(effects, player, STATUS_OPO_OPO_FORM)

    return effects
end
