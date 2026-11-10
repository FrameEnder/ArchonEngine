-- Fire II (Lv. 18): the target and enemies near it.
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- Also used by the masked mage in a quest battle (recorded_amounts there).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 100
local POTENCY = 80
local MP_COST = 1500
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmFireCost(player, MP_COST))
    blmFireStep(effects, player)

    return effects
end
