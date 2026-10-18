-- Blizzard II (Lv. 12), the player one: the target and enemies near it.
-- NOTE: Astral Fire and Umbral Ice (the job gauge) aren't implemented: no potency or MP changes from them. Since 7.x Thunder spells need Thunderhead; without the gauge, changing element (this) grants it.
-- (Action 146 is the older Blizzard II that NPCs use.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 80
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    effects:gain_effect_self(EFFECT_THUNDERHEAD, 0, THUNDERHEAD_DURATION)

    return effects
end
