-- Scathe (Lv. 15): 20% chance of double potency.
-- Also used by Papalymo in a quest battle (recorded_amounts there).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 100
local POTENCY = 100
local DOUBLE_CHANCE = 0.2

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, NPC_POTENCY))
        return effects
    end

    local potency = POTENCY
    if math.random() < DOUBLE_CHANCE then
        potency = POTENCY * 2
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, potency, TRAITS_THM_MAIM_AND_MEND)))

    return effects
end
