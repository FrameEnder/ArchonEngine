-- Freeze (Lv. 40): the target and enemies near it.
-- NOTE: Astral Fire and Umbral Ice (the job gauge) aren't implemented: no potency or MP changes from them. Since 7.x Thunder spells need Thunderhead; without the gauge, changing element (this) grants it.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 120

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND)))

    return effects
end
