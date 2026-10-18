-- Fast Blade (Lv. 1)
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 150
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
