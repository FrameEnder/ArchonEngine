-- Freeze (Lv. 40): the target and enemies near it.
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- Only under Umbral Ice (the client's): free there, and its MP back as an Ice spell; three Umbral Hearts from 58.
local UMBRAL_HEART_LEVEL = 58
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 120

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmIcePotency(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    if blmElement(player) < 0 then
        effects:set_mp_cost(0)
        blmSetElement(effects, player, blmElement(player), true)
        if player.level >= UMBRAL_HEART_LEVEL then
            effects:gauge_set("umbral_hearts", 3)
        end
    end

    return effects
end
