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
    -- With Umbral Hearts (all spent) it costs two thirds of the MP left; three Astral Soul from 100.
    if player:gauge("umbral_hearts") > 0 then
        effects:set_mp_cost(math.max(math.floor(player.mp * 2 / 3), MIN_MP_COST))
        effects:gauge_set("umbral_hearts", 0)
    else
        effects:set_mp_cost(math.max(player.mp, MIN_MP_COST))
    end
    if player.level >= 100 then
        effects:gauge_add("astral_soul", 3)
    end
    blmSetElement(effects, player, 3, false)

    return effects
end
