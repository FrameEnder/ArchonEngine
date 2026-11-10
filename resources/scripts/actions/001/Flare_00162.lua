-- Flare (Lv. 50): the target and enemies near it, 30% less after the first.
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- Only under Astral Fire (the client's); it spends all the MP left (at least 800) and gives Astral Fire III.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 240
local FALLOFF = 30
local MIN_MP_COST = 800

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(FALLOFF)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(math.max(player.mp, MIN_MP_COST))
    blmSetElement(effects, player, 3, false)

    return effects
end
