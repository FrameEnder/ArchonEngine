-- True Strike (Lv. 4): changes to coeurl form.
-- Raptor's Fury (the gauge's "raptor_fury") adds 200.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 260
local POTENCY = 270
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 2)
    end
    mnkUsePerfectBalance(effects, player)
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        mnkShift(effects, player, STATUS_COEURL_FORM)
        return effects
    end

    local potency = POTENCY
    -- Raptor's Fury: 200 more (Twin Snakes gives it.)
    if mnkSpendFury(effects, player, "raptor_fury") then
        potency = potency + 200
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_COEURL_FORM)

    return effects
end
