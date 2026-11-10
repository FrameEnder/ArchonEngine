-- Bootshine (Lv. 1): changes to raptor form.
-- Opo-opo's Fury (the gauge's "opo_fury") adds 200; in Opo-opo form it's surely critical.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 180
local POTENCY = 190
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 1)
    end
    mnkUsePerfectBalance(effects, player)
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        mnkShift(effects, player, STATUS_RAPTOR_FORM)
        return effects
    end

    local potency = POTENCY
    -- Opo-opo's Fury: 200 more (Dragon Kick gives it); in Opo-opo form a sure critical.
    if mnkSpendFury(effects, player, "opo_fury") then
        potency = potency + 200
    end
    if mnkInOpoOpo(player) then
        effects:guarantee_critical()
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, potency))
    mnkShift(effects, player, STATUS_RAPTOR_FORM)

    return effects
end
