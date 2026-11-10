-- Snap Punch (Lv. 6): changes to opo-opo form.
-- Coeurl's Fury (the gauge's "coeurl_fury") adds 150. NOTE: the flank bonus (330) isn't implemented (no positionals).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 230
local POTENCY = 240
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 3)
    end
    mnkUsePerfectBalance(effects, player)
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        mnkShift(effects, player, STATUS_OPO_OPO_FORM)
        return effects
    end

    local potency = POTENCY
    -- Coeurl's Fury: 150 more (Demolish gives two.)
    if mnkSpendFury(effects, player, "coeurl_fury") then
        potency = potency + 150
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_OPO_OPO_FORM)

    return effects
end
