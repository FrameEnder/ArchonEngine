-- Fire (Lv. 2): 40% chance of Firestarter (Lv. 42).
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 180
local POTENCY = 180
local MP_COST = 800
local TRAIT_FIRESTARTER = 32
local EFFECT_FIRESTARTER = 165
local FIRESTARTER_CHANCE = 0.4
local FIRESTARTER_DURATION = 0.0 -- no time limit on retail
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
    blmUseHeart(effects, player)
    blmFireStep(effects, player)
    if player:has_trait(TRAIT_FIRESTARTER) and math.random() < FIRESTARTER_CHANCE then
        effects:gain_effect_self(EFFECT_FIRESTARTER, 0, FIRESTARTER_DURATION)
    end

    return effects
end
